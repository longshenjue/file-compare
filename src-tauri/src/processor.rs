use crate::models::*;
use anyhow::{Context, Result};
use duckdb::Connection;
use std::collections::HashMap;

pub struct DataProcessor {
    conn: Connection,
}

impl DataProcessor {
    pub fn new() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Ok(Self { conn })
    }

    /// 创建空表（用于 Double Check 功能）
    pub fn create_empty_tables(&self) -> Result<()> {
        self.conn.execute("DROP TABLE IF EXISTS source_a", [])?;
        self.conn.execute("DROP TABLE IF EXISTS source_b", [])?;
        // 不创建表结构，让 load_historical_data 根据数据自动创建
        Ok(())
    }

    pub fn read_csv_headers(&self, file_path: &str, header_row: usize) -> Result<Vec<String>> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(header_row > 0)
            .from_path(file_path)
            .context("无法打开CSV文件")?;

        // Skip to header row if needed
        let mut iter = reader.records();
        for _ in 0..header_row.saturating_sub(1) {
            iter.next();
        }

        let headers = reader.headers()?;
        let header_vec: Vec<String> = headers.iter().map(|h| h.to_string()).collect();
        
        Ok(header_vec)
    }

    pub fn load_csv_to_table(
        &self,
        file_path: &str,
        table_name: &str,
        header_row: usize,
    ) -> Result<()> {
        // Create table and load CSV using DuckDB's native CSV reader
        let sql = format!(
            "CREATE TABLE {} AS SELECT * FROM read_csv_auto('{}', header={}, delim=',', quote='\"')",
            table_name, file_path, header_row > 0
        );
        
        self.conn.execute(&sql, [])?;
        Ok(())
    }

    /// 将历史数据加载到现有表中
    pub fn load_historical_data(
        &self,
        table_name: &str,
        historical_records: Vec<HashMap<String, serde_json::Value>>,
    ) -> Result<()> {
        if historical_records.is_empty() {
            return Ok(());
        }

        // 如果表不存在，先创建表（根据第一条记录的结构）
        let first_record = &historical_records[0];
        let columns: Vec<String> = first_record.keys().map(|k| format!("\"{}\"", k)).collect();
        
        // 检查表是否存在
        let table_exists: bool = self.conn
            .query_row(
                "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = ?",
                [table_name],
                |row| Ok(row.get::<_, i64>(0)? > 0),
            )
            .unwrap_or(false);

        if !table_exists {
            // 根据第一条记录创建表结构
            let column_defs: Vec<String> = columns
                .iter()
                .map(|col| {
                    // 简化处理：所有字段都使用 VARCHAR 类型
                    format!("{} VARCHAR", col)
                })
                .collect();
            
            let create_sql = format!(
                "CREATE TABLE {} ({})",
                table_name,
                column_defs.join(", ")
            );
            self.conn.execute(&create_sql, [])?;
        }

        // 将历史数据插入到表中
        for record in historical_records {
            let columns: Vec<String> = record.keys().map(|k| format!("\"{}\"", k)).collect();
            let values: Vec<String> = record
                .values()
                .map(|v| match v {
                    serde_json::Value::String(s) => format!("'{}'", s.replace("'", "''")),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Null => "NULL".to_string(),
                    _ => "NULL".to_string(),
                })
                .collect();

            let insert_sql = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                table_name,
                columns.join(", "),
                values.join(", ")
            );

            let _ = self.conn.execute(&insert_sql, []);
        }

        Ok(())
    }

    /// 获取表中的所有数据
    pub fn get_table_data(&self, table_name: &str) -> Result<Vec<HashMap<String, serde_json::Value>>> {
        let sql = format!("SELECT * FROM {}", table_name);
        self.execute_query_to_json(&sql)
    }

    /// 获取表中的所有列名
    pub fn get_table_columns(&self, table_name: &str) -> Result<Vec<String>> {
        let sql = format!(
            "SELECT column_name FROM information_schema.columns WHERE table_name = '{}' ORDER BY ordinal_position",
            table_name
        );
        
        let mut stmt = self.conn.prepare(&sql)?;
        let columns: Vec<String> = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .filter_map(|r| r.ok())
            .collect();
        
        Ok(columns)
    }

    pub fn apply_data_cleaning(
        &self,
        table_name: &str,
        mappings: &[ColumnMapping],
    ) -> Result<()> {
        for mapping in mappings {
            let mut column_expr = format!("\"{}\"", mapping.source_column);
            
            // Apply format rules
            for rule in &mapping.format_rules {
                column_expr = self.apply_format_rule(&column_expr, rule)?;
            }
            
            // Add new column with cleaned data
            let alter_sql = format!(
                "ALTER TABLE {} ADD COLUMN \"{}\" VARCHAR",
                table_name, mapping.field_name
            );
            let _ = self.conn.execute(&alter_sql, []);
            
            let update_sql = format!(
                "UPDATE {} SET \"{}\" = {}",
                table_name, mapping.field_name, column_expr
            );
            self.conn.execute(&update_sql, [])?;
        }
        
        Ok(())
    }

    fn apply_format_rule(&self, column_expr: &str, rule: &FormatRule) -> Result<String> {
        let expr = match rule.operation.as_str() {
            "DEL_PRE" => {
                // Delete first N characters
                let n: i32 = rule.value.parse().unwrap_or(0);
                format!("SUBSTRING({}, {} + 1)", column_expr, n)
            }
            "DEL_AFTER" => {
                // Keep only first N characters
                let n: i32 = rule.value.parse().unwrap_or(0);
                format!("LEFT({}, {})", column_expr, n)
            }
            "DEL_CHAR" => {
                // Remove specific character
                format!("REPLACE({}, '{}', '')", column_expr, rule.value)
            }
            "REPLACE_TWO_CHAR" => {
                // Replace character (format: "from,to")
                let parts: Vec<&str> = rule.value.split(',').collect();
                if parts.len() == 2 {
                    format!("REPLACE({}, '{}', '{}')", column_expr, parts[0], parts[1])
                } else {
                    column_expr.to_string()
                }
            }
            "BRA_VALUE" => {
                // Extract value between brackets
                format!("REGEXP_EXTRACT({}, '\\[(.+?)\\]', 1)", column_expr)
            }
            "DIVIDE_NUMBER" => {
                // Divide by a number
                let n: f64 = rule.value.parse().unwrap_or(1.0);
                format!("CAST({} AS DOUBLE) / {}", column_expr, n)
            }
            "ABS_VALUE" => {
                // Absolute value
                format!("ABS(CAST({} AS DOUBLE))", column_expr)
            }
            "ADD_CHAR_PRE" => {
                // Add characters before
                format!("CONCAT('{}', {})", rule.value, column_expr)
            }
            "ADD_CHAR_AFTER" => {
                // Add characters after
                format!("CONCAT({}, '{}')", column_expr, rule.value)
            }
            "XENDIT_TIME" => {
                // Convert Xendit timestamp format
                format!("STRPTIME({}, '%Y-%m-%dT%H:%M:%S')", column_expr)
            }
            _ => column_expr.to_string(),
        };
        
        Ok(expr)
    }

    pub fn remove_duplicates(&self, table_name: &str, id_field: &str) -> Result<()> {
        let sql = format!(
            "DELETE FROM {} WHERE rowid NOT IN (SELECT MIN(rowid) FROM {} GROUP BY \"{}\")",
            table_name, table_name, id_field
        );
        self.conn.execute(&sql, [])?;
        Ok(())
    }

    pub fn normalize_status(
        &self,
        table_name: &str,
        status_field: &str,
        mappings: &[StatusMapping],
    ) -> Result<()> {
        // Add normalized status column
        let alter_sql = format!(
            "ALTER TABLE {} ADD COLUMN normalized_status VARCHAR",
            table_name
        );
        let _ = self.conn.execute(&alter_sql, []);
        
        // Apply status mappings
        for mapping in mappings {
            if !mapping.source_status.is_empty() {
                let status_list = mapping.source_status
                    .iter()
                    .map(|s| format!("'{}'", s))
                    .collect::<Vec<_>>()
                    .join(", ");
                
                let update_sql = format!(
                    "UPDATE {} SET normalized_status = '{}' WHERE \"{}\" IN ({})",
                    table_name, mapping.target_status, status_field, status_list
                );
                self.conn.execute(&update_sql, [])?;
            }
        }
        
        Ok(())
    }

    pub fn perform_reconciliation(
        &self,
        order_id_field: &str,
        bank_id_field: &str,
        order_amount_field: &str,
        bank_amount_field: &str,
    ) -> Result<ReconciliationResult> {
        // Matched records (status and amount both match)
        let matched_sql = if !order_amount_field.is_empty() && !bank_amount_field.is_empty() {
            format!(
                "SELECT a.*, b.* FROM source_a a INNER JOIN source_b b ON a.\"{}\" = b.\"{}\" WHERE a.normalized_status = b.normalized_status AND CAST(a.\"{}\" AS DOUBLE) = CAST(b.\"{}\" AS DOUBLE)",
                order_id_field, bank_id_field, order_amount_field, bank_amount_field
            )
        } else {
            format!(
                "SELECT a.*, b.* FROM source_a a INNER JOIN source_b b ON a.\"{}\" = b.\"{}\" WHERE a.normalized_status = b.normalized_status",
                order_id_field, bank_id_field
            )
        };
        let matched = self.execute_query_to_json(&matched_sql)?;

        // Records with amount difference
        let diff_amount_sql = if !order_amount_field.is_empty() && !bank_amount_field.is_empty() {
            format!(
                "SELECT a.*, b.* FROM source_a a INNER JOIN source_b b ON a.\"{}\" = b.\"{}\" WHERE a.normalized_status = b.normalized_status AND CAST(a.\"{}\" AS DOUBLE) != CAST(b.\"{}\" AS DOUBLE)",
                order_id_field, bank_id_field, order_amount_field, bank_amount_field
            )
        } else {
            // 如果没有金额字段，返回空结果
            "SELECT NULL LIMIT 0".to_string()
        };
        let diff_amount = self.execute_query_to_json(&diff_amount_sql)?;

        // Only in source_a
        let only_in_a_sql = format!(
            "SELECT a.* FROM source_a a LEFT JOIN source_b b ON a.\"{}\" = b.\"{}\" WHERE b.\"{}\" IS NULL",
            order_id_field, bank_id_field, bank_id_field
        );
        let only_in_a = self.execute_query_to_json(&only_in_a_sql)?;

        // Only in source_b
        let only_in_b_sql = format!(
            "SELECT b.* FROM source_b b LEFT JOIN source_a a ON b.\"{}\" = a.\"{}\" WHERE a.\"{}\" IS NULL",
            bank_id_field, order_id_field, order_id_field
        );
        let only_in_b = self.execute_query_to_json(&only_in_b_sql)?;

        Ok(ReconciliationResult {
            matched,
            only_in_a,
            only_in_b,
            diff_amount,
        })
    }

    fn execute_query_to_json(&self, sql: &str) -> Result<Vec<HashMap<String, serde_json::Value>>> {
        // 使用临时表方式避免DuckDB的列名问题
        // 先将查询结果存入临时表，再从临时表读取
        let temp_table_name = format!("temp_result_{}", chrono::Utc::now().timestamp_millis());
        
        let create_temp_sql = format!("CREATE TEMP TABLE {} AS {}", temp_table_name, sql);
        self.conn.execute(&create_temp_sql, [])?;
        
        // 获取列名
        let columns_sql = format!(
            "SELECT column_name FROM information_schema.columns WHERE table_name = '{}' ORDER BY ordinal_position",
            temp_table_name
        );
        
        let column_names: Vec<String> = {
            let mut column_stmt = self.conn.prepare(&columns_sql)?;
            let mapped_rows = column_stmt.query_map([], |row| row.get::<_, String>(0))?;
            mapped_rows.filter_map(|r| r.ok()).collect()
        };
        
        // 如果没有列名，清理并返回空
        if column_names.is_empty() {
            let _ = self.conn.execute(&format!("DROP TABLE {}", temp_table_name), []);
            return Ok(Vec::new());
        }
        
        // 读取数据
        let select_sql = format!("SELECT * FROM {}", temp_table_name);
        let result = {
            let mut stmt = self.conn.prepare(&select_sql)?;
            let mut rows = stmt.query([])?;
            
            let mut data = Vec::new();
            while let Some(row) = rows.next()? {
                let mut map = HashMap::new();
                for (i, name) in column_names.iter().enumerate() {
                    let json_value = if let Ok(v) = row.get::<_, String>(i) {
                        serde_json::Value::String(v)
                    } else if let Ok(v) = row.get::<_, i64>(i) {
                        serde_json::Value::Number(v.into())
                    } else if let Ok(v) = row.get::<_, f64>(i) {
                        serde_json::Value::Number(
                            serde_json::Number::from_f64(v).unwrap_or(serde_json::Number::from(0))
                        )
                    } else if let Ok(v) = row.get::<_, bool>(i) {
                        serde_json::Value::Bool(v)
                    } else {
                        serde_json::Value::Null
                    };
                    map.insert(name.clone(), json_value);
                }
                data.push(map);
            }
            data
        };
        
        // 清理临时表
        let _ = self.conn.execute(&format!("DROP TABLE {}", temp_table_name), []);
        
        Ok(result)
    }
}

