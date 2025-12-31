use anyhow::{Context, Result};
use chrono::NaiveDate;
use duckdb::{Connection, params, params_from_iter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use log;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderFile {
    pub file_id: String,
    pub file_name: String,
    pub config_id: String,
    pub config_name: String,
    pub source_name: String, // 改为通用的 source_name
    pub upload_time: String,
    pub record_count: usize,
    pub date_range: Option<DateRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryCondition {
    pub field: String,
    pub operator: String, // "equals", "contains", "gt", "lt", "between"
    pub value: String,
    pub value2: Option<String>, // for "between"
}

pub struct OrderManager {
    conn: Connection,
}

impl OrderManager {
    pub fn new() -> Result<Self> {
        let orders_dir = dirs::data_dir()
            .context("无法获取数据目录")
            .map_err(|e| {
                log::error!("Failed to get data directory: {}", e);
                e
            })?
            .join(".file-compare")
            .join("orders");

        log::info!("Orders directory: {:?}", orders_dir);

        fs::create_dir_all(&orders_dir)
            .with_context(|| format!("Failed to create orders directory: {:?}", orders_dir))
            .map_err(|e| {
                log::error!("{}", e);
                e
            })?;

        // 创建持久化的 DuckDB 数据库
        let db_path = orders_dir.join("orders.duckdb");
        log::info!("Opening database: {:?}", db_path);
        
        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open database: {:?}", db_path))
            .map_err(|e| {
                log::error!("{}", e);
                e
            })?;

        // 初始化表结构
        conn.execute(
            "CREATE TABLE IF NOT EXISTS order_data (
                record_id VARCHAR PRIMARY KEY,
                config_id VARCHAR NOT NULL,
                config_name VARCHAR NOT NULL,
                source_name VARCHAR NOT NULL,
                upload_date DATE NOT NULL,
                upload_time TIMESTAMP NOT NULL,
                file_name VARCHAR NOT NULL,
                data JSON NOT NULL
            )",
            [],
        )
        .context("Failed to create order_data table")
        .map_err(|e| {
            log::error!("{}", e);
            e
        })?;

        // 创建索引以优化查询
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_config_source_date 
             ON order_data(config_id, source_name, upload_date)",
            [],
        )
        .context("Failed to create index")
        .map_err(|e| {
            log::error!("{}", e);
            e
        })?;

        // 创建元数据表，记录上传历史
        conn.execute(
            "CREATE TABLE IF NOT EXISTS upload_metadata (
                file_id VARCHAR PRIMARY KEY,
                config_id VARCHAR NOT NULL,
                config_name VARCHAR NOT NULL,
                source_name VARCHAR NOT NULL,
                file_name VARCHAR NOT NULL,
                upload_time TIMESTAMP NOT NULL,
                record_count INTEGER NOT NULL
            )",
            [],
        )
        .context("Failed to create upload_metadata table")
        .map_err(|e| {
            log::error!("{}", e);
            e
        })?;

        log::info!("OrderManager initialized successfully");
        Ok(Self { conn })
    }

    /// 保存订单数据到 DuckDB（带去重逻辑）
    pub fn save_order_data(
        &self,
        config_id: &str,
        config_name: &str,
        source_name: &str,
        file_name: &str,
        upload_date: &str, // YYYY-MM-DD
        records: Vec<HashMap<String, serde_json::Value>>,
    ) -> Result<OrderFile> {
        let file_id = uuid::Uuid::new_v4().to_string();
        let upload_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // 去重：检查是否已经存在相同的 config_id + source_name + upload_date 的数据
        let existing_count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM order_data 
                 WHERE config_id = ? AND source_name = ? AND upload_date = ?",
                [config_id, source_name, upload_date],
                |row| row.get(0),
            )?;

        if existing_count > 0 {
            // 删除旧数据
            self.conn.execute(
                "DELETE FROM order_data 
                 WHERE config_id = ? AND source_name = ? AND upload_date = ?",
                [config_id, source_name, upload_date],
            )?;
        }

        // 插入新数据
        let mut stmt = self.conn.prepare(
            "INSERT INTO order_data 
             (record_id, config_id, config_name, source_name, upload_date, upload_time, file_name, data) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )?;

        for record in &records {
            let record_id = uuid::Uuid::new_v4().to_string();
            let data_json = serde_json::to_string(record)?;

            stmt.execute([
                record_id.as_str(),
                config_id,
                config_name,
                source_name,
                upload_date,
                &upload_time,
                file_name,
                &data_json,
            ])?;
        }

        // 保存元数据
        self.conn.execute(
            "INSERT INTO upload_metadata 
             (file_id, config_id, config_name, source_name, file_name, upload_time, record_count) 
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            [
                file_id.as_str(),
                config_id,
                config_name,
                source_name,
                file_name,
                &upload_time,
                &records.len().to_string(),
            ],
        )?;

        Ok(OrderFile {
            file_id,
            file_name: file_name.to_string(),
            config_id: config_id.to_string(),
            config_name: config_name.to_string(),
            source_name: source_name.to_string(),
            upload_time,
            record_count: records.len(),
            date_range: Some(DateRange {
                start: upload_date.to_string(),
                end: upload_date.to_string(),
            }),
        })
    }

    /// 获取所有上传文件的元数据列表
    pub fn list_order_files(&self) -> Result<Vec<OrderFile>> {
        log::info!("Listing order files from database");
        
        let mut stmt = self.conn.prepare(
            "SELECT file_id, config_id, config_name, source_name, file_name, 
                    strftime(upload_time, '%Y-%m-%d %H:%M:%S') as upload_time_str, 
                    record_count 
             FROM upload_metadata 
             ORDER BY upload_time DESC",
        )
        .context("Failed to prepare list_order_files query")
        .map_err(|e| {
            log::error!("{}", e);
            e
        })?;

        let files = stmt
            .query_map([], |row| {
                Ok(OrderFile {
                    file_id: row.get(0)?,
                    config_id: row.get(1)?,
                    config_name: row.get(2)?,
                    source_name: row.get(3)?,
                    file_name: row.get(4)?,
                    upload_time: row.get(5)?,
                    record_count: row.get(6)?,
                    date_range: None,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(files)
    }

    /// 按条件查询订单数据（跨所有文件）
    pub fn query_orders(
        &self,
        config_id: Option<&str>,
        source_name: Option<&str>,
        conditions: Vec<QueryCondition>,
        limit: Option<usize>,
    ) -> Result<Vec<HashMap<String, serde_json::Value>>> {
        log::info!("Querying orders: config_id={:?}, source_name={:?}, conditions={}, limit={:?}",
                   config_id, source_name, conditions.len(), limit);
        
        // 构建基础查询
        let mut sql = String::from("SELECT data FROM order_data WHERE 1=1");
        let mut param_values: Vec<&str> = Vec::new();

        if let Some(cid) = config_id {
            sql.push_str(" AND config_id = ?");
            param_values.push(cid);
        }

        if let Some(sname) = source_name {
            sql.push_str(" AND source_name = ?");
            param_values.push(sname);
        }

        // 执行查询
        let mut stmt = self.conn.prepare(&sql)?;
        
        let rows = stmt.query_map(params_from_iter(param_values.iter().copied()), |row| {
            let data_str: String = row.get(0)?;
            Ok(data_str)
        })?;

        let mut results: Vec<HashMap<String, serde_json::Value>> = Vec::new();
        for row_result in rows {
            let data_str = row_result?;
            let record: HashMap<String, serde_json::Value> = serde_json::from_str(&data_str)?;
            results.push(record);
        }

        // 应用内存过滤条件
        for condition in conditions {
            results.retain(|record| {
                if let Some(value) = record.get(&condition.field) {
                    match condition.operator.as_str() {
                        "equals" => {
                            let record_val = value.as_str().unwrap_or("");
                            record_val == condition.value
                        }
                        "contains" => {
                            let record_val = value.as_str().unwrap_or("");
                            record_val.contains(&condition.value)
                        }
                        "gt" => {
                            if let (Some(v1), Ok(v2)) =
                                (value.as_f64(), condition.value.parse::<f64>())
                            {
                                v1 > v2
                            } else {
                                false
                            }
                        }
                        "lt" => {
                            if let (Some(v1), Ok(v2)) =
                                (value.as_f64(), condition.value.parse::<f64>())
                            {
                                v1 < v2
                            } else {
                                false
                            }
                        }
                        "between" => {
                            if let Some(v1) = value.as_f64() {
                                if let Ok(v2) = condition.value.parse::<f64>() {
                                    if let Some(v3_str) = &condition.value2 {
                                        if let Ok(v3) = v3_str.parse::<f64>() {
                                            return v1 >= v2 && v1 <= v3;
                                        }
                                    }
                                }
                            }
                            false
                        }
                        _ => true,
                    }
                } else {
                    false
                }
            });
        }

        // 应用限制
        if let Some(limit) = limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    /// 查询指定日期范围的历史数据（用于对账）
    pub fn load_historical_data(
        &self,
        config_id: &str,
        source_name: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<Vec<HashMap<String, serde_json::Value>>> {
        let mut stmt = self.conn.prepare(
            "SELECT data FROM order_data 
             WHERE config_id = ? AND source_name = ? 
             AND upload_date BETWEEN ? AND ?",
        )?;

        let rows = stmt.query_map([config_id, source_name, start_date, end_date], |row| {
            let data_str: String = row.get(0)?;
            Ok(data_str)
        })?;

        let mut results = Vec::new();
        for row_result in rows {
            let data_str = row_result?;
            let record: HashMap<String, serde_json::Value> = serde_json::from_str(&data_str)?;
            results.push(record);
        }

        Ok(results)
    }

    /// 删除指定文件的数据
    pub fn delete_order_file(&self, file_id: &str) -> Result<()> {
        // 从元数据获取信息，使用strftime转换TIMESTAMP
        let metadata_result = self.conn.query_row(
            "SELECT config_id, source_name, strftime(upload_time, '%Y-%m-%d %H:%M:%S') 
             FROM upload_metadata WHERE file_id = ?",
            [file_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?)),
        );

        if let Ok((config_id, source_name, upload_time)) = metadata_result {
            // 解析 upload_time 获取日期
            let upload_date = upload_time.split(' ').next().unwrap_or("");

            // 删除对应的数据
            self.conn.execute(
                "DELETE FROM order_data 
                 WHERE config_id = ? AND source_name = ? AND upload_date = ?",
                params![&config_id, &source_name, upload_date],
            )?;

            // 删除元数据
            self.conn.execute(
                "DELETE FROM upload_metadata WHERE file_id = ?",
                params![file_id],
            )?;
        }

        Ok(())
    }

    /// 清除指定日期之前的历史订单
    pub fn cleanup_orders_before_date(&self, before_date: &str) -> Result<usize> {
        // 验证日期格式
        NaiveDate::parse_from_str(before_date, "%Y-%m-%d").context("日期格式错误")?;

        // 删除数据
        self.conn.execute(
            "DELETE FROM order_data WHERE upload_date < ?",
            params![before_date],
        )?;

        // 删除元数据
        let deleted_metadata = self.conn.execute(
            "DELETE FROM upload_metadata WHERE DATE(upload_time) < ?",
            params![before_date],
        )?;

        Ok(deleted_metadata)
    }

    /// 清除所有订单数据
    pub fn clear_all_orders(&self) -> Result<usize> {
        // 删除所有数据
        self.conn.execute("DELETE FROM order_data", [])?;

        // 删除所有元数据
        let deleted_metadata = self.conn.execute("DELETE FROM upload_metadata", [])?;

        Ok(deleted_metadata)
    }

    /// 获取配置相关的订单文件
    pub fn get_files_by_config(&self, config_id: &str) -> Result<Vec<OrderFile>> {
        let files = self.list_order_files()?;
        Ok(files
            .into_iter()
            .filter(|f| f.config_id == config_id)
            .collect())
    }

}
