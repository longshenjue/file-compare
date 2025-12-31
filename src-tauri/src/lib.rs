mod models;
mod processor;
mod exporter;
mod config_manager;
mod task_manager;
mod order_manager;
mod logger;

use models::*;
use processor::DataProcessor;
use exporter::export_to_csv;
use config_manager::ConfigManager;
use task_manager::TaskManager;
use order_manager::{OrderManager, OrderFile, QueryCondition};

#[tauri::command]
fn read_csv_headers(file_path: String, header_row: usize) -> Result<Vec<String>, String> {
    let processor = DataProcessor::new().map_err(|e| e.to_string())?;
    processor
        .read_csv_headers(&file_path, header_row)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn reconcile(
    source_a_config: FileConfig,
    source_b_config: FileConfig,
    source_a_mappings: Vec<ColumnMapping>,
    source_b_mappings: Vec<ColumnMapping>,
    match_config: MatchConfig,
    config_id: String,
    config_name: String,
    task_name: String,
) -> Result<(ReconciliationTask, ReconciliationResult), String> {
    log::info!("Starting reconciliation: task_name={}, config_id={}", task_name, config_id);
    
    let processor = DataProcessor::new()
        .map_err(|e| {
            let err_msg = format!("Failed to create DataProcessor: {}", e);
            log::error!("{}", err_msg);
            err_msg
        })?;
    
    let order_manager = OrderManager::new()
        .map_err(|e| {
            let err_msg = format!("Failed to create OrderManager: {}", e);
            log::error!("{}", err_msg);
            err_msg
        })?;

    // Step 1: 加载 CSV 文件并保存到 DuckDB
    processor
        .load_csv_to_table(&source_a_config.file_path, "source_a", source_a_config.header)
        .map_err(|e| format!("加载数据源A文件失败: {}", e))?;

    processor
        .load_csv_to_table(&source_b_config.file_path, "source_b", source_b_config.header)
        .map_err(|e| format!("加载数据源B文件失败: {}", e))?;

    // Step 2: 数据清洗
    processor
        .apply_data_cleaning("source_a", &source_a_mappings)
        .map_err(|e| format!("清洗数据源A失败: {}", e))?;

    processor
        .apply_data_cleaning("source_b", &source_b_mappings)
        .map_err(|e| format!("清洗数据源B失败: {}", e))?;

    // Step 3: 去重
    if source_a_config.remove_duplicate {
        processor
            .remove_duplicates("source_a", &match_config.source_a_id_field)
            .map_err(|e| format!("去重数据源A失败: {}", e))?;
    }

    if source_b_config.remove_duplicate {
        processor
            .remove_duplicates("source_b", &match_config.source_b_id_field)
            .map_err(|e| format!("去重数据源B失败: {}", e))?;
    }

    // Step 4: 状态标准化
    let source_a_status_field = source_a_mappings
        .iter()
        .find(|m| m.field_type == "OrderStatus")
        .map(|m| m.field_name.clone())
        .unwrap_or_default();

    let source_b_status_field = source_b_mappings
        .iter()
        .find(|m| m.field_type == "OrderStatus")
        .map(|m| m.field_name.clone())
        .unwrap_or_default();

    let source_a_amount_field = source_a_mappings
        .iter()
        .find(|m| m.field_type == "OrderAmount")
        .map(|m| m.field_name.clone())
        .unwrap_or_default();

    let source_b_amount_field = source_b_mappings
        .iter()
        .find(|m| m.field_type == "OrderAmount")
        .map(|m| m.field_name.clone())
        .unwrap_or_default();

    if !source_a_status_field.is_empty() {
        processor
            .normalize_status("source_a", &source_a_status_field, &match_config.source_a_status_mapping)
            .map_err(|e| format!("标准化数据源A状态失败: {}", e))?;
    }

    if !source_b_status_field.is_empty() {
        processor
            .normalize_status("source_b", &source_b_status_field, &match_config.source_b_status_mapping)
            .map_err(|e| format!("标准化数据源B状态失败: {}", e))?;
    }

    // Step 5: 获取清洗后的数据准备保存到 DuckDB
    let current_source_a = processor
        .get_table_data("source_a")
        .map_err(|e| format!("获取数据源A失败: {}", e))?;
    
    let current_source_b = processor
        .get_table_data("source_b")
        .map_err(|e| format!("获取数据源B失败: {}", e))?;

    // Step 6: 保存到 DuckDB（带去重）
    let upload_date = &source_a_config.date_range.start;
    
    order_manager
        .save_order_data(
            &config_id,
            &config_name,
            &source_a_config.source_name,
            &source_a_config.file_name,
            upload_date,
            current_source_a.clone(),
        )
        .map_err(|e| format!("保存数据源A到数据库失败: {}", e))?;

    order_manager
        .save_order_data(
            &config_id,
            &config_name,
            &source_b_config.source_name,
            &source_b_config.file_name,
            upload_date,
            current_source_b.clone(),
        )
        .map_err(|e| format!("保存数据源B到数据库失败: {}", e))?;

    // Step 7: 如果启用历史数据，从 DuckDB 加载
    if match_config.use_historical_source_a {
        let start_date = chrono::NaiveDate::parse_from_str(upload_date, "%Y-%m-%d")
            .map_err(|e| e.to_string())?
            .checked_sub_signed(chrono::Duration::days(match_config.history_days as i64))
            .ok_or("日期计算失败")?
            .format("%Y-%m-%d")
            .to_string();

        let historical_data = order_manager
            .load_historical_data(
                &config_id,
                &source_a_config.source_name,
                &start_date,
                upload_date,
            )
            .map_err(|e| format!("加载历史数据源A失败: {}", e))?;
        
        if !historical_data.is_empty() {
            processor
                .load_historical_data("source_a", historical_data)
                .map_err(|e| format!("导入历史数据源A失败: {}", e))?;
        }
    }

    if match_config.use_historical_source_b {
        let start_date = chrono::NaiveDate::parse_from_str(upload_date, "%Y-%m-%d")
            .map_err(|e| e.to_string())?
            .checked_sub_signed(chrono::Duration::days(match_config.history_days as i64))
            .ok_or("日期计算失败")?
            .format("%Y-%m-%d")
            .to_string();

        let historical_data = order_manager
            .load_historical_data(
                &config_id,
                &source_b_config.source_name,
                &start_date,
                upload_date,
            )
            .map_err(|e| format!("加载历史数据源B失败: {}", e))?;
        
        if !historical_data.is_empty() {
            processor
                .load_historical_data("source_b", historical_data)
                .map_err(|e| format!("导入历史数据源B失败: {}", e))?;
        }
    }

    // Step 8: 执行对账
    let result = processor
        .perform_reconciliation(
            &match_config.source_a_id_field, 
            &match_config.source_b_id_field,
            &source_a_amount_field,
            &source_b_amount_field
        )
        .map_err(|e| format!("对账失败: {}", e))?;

    // Step 9: 创建任务记录
    let task_id = format!("task_{}", chrono::Utc::now().timestamp_millis());
    let created_at = chrono::Utc::now().to_rfc3339();

    let task = ReconciliationTask {
        task_id: task_id.clone(),
        task_name,
        config_id: config_id.clone(),
        config_name: config_name.clone(),
        source_a_name: source_a_config.source_name.clone(),
        source_b_name: source_b_config.source_name.clone(),
        task_type: source_a_config.file_type.clone(),
        date_range: source_a_config.date_range.clone(),
        created_at,
        source_a_file_name: source_a_config.file_name.clone(),
        source_b_file_name: source_b_config.file_name.clone(),
        stats: ReconciliationStats {
            matched_count: result.matched.len(),
            only_in_source_a_count: result.only_in_a.len(),
            only_in_source_b_count: result.only_in_b.len(),
            diff_amount_count: result.diff_amount.len(),
            total_source_a: result.matched.len() + result.only_in_a.len() + result.diff_amount.len(),
            total_source_b: result.matched.len() + result.only_in_b.len() + result.diff_amount.len(),
        },
        used_historical_source_a: match_config.use_historical_source_a,
        used_historical_source_b: match_config.use_historical_source_b,
    };

    // Step 10: 保存任务
    let task_manager = TaskManager::new().map_err(|e| e.to_string())?;
    task_manager
        .save_task(&task, &result)
        .map_err(|e| format!("保存任务失败: {}", e))?;

    Ok((task, result))
}

#[tauri::command]
fn export_results(
    results: ReconciliationResult,
    export_type: String,
    file_path: String,
) -> Result<(), String> {
    export_to_csv(&results, &export_type, &file_path).map_err(|e| e.to_string())
}

// 配置管理命令
#[tauri::command]
fn load_configs() -> Result<Vec<ChannelConfig>, String> {
    let manager = ConfigManager::new().map_err(|e| e.to_string())?;
    manager.load_configs().map_err(|e| e.to_string())
}

#[tauri::command]
fn save_config(config: ChannelConfig) -> Result<(), String> {
    let manager = ConfigManager::new().map_err(|e| e.to_string())?;
    let mut configs = manager.load_configs().map_err(|e| e.to_string())?;
    
    // 查找是否存在相同ID的配置
    if let Some(index) = configs.iter().position(|c| c.id == config.id) {
        configs[index] = config;
    } else {
        configs.push(config);
    }
    
    manager.save_configs(&configs).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_config(config_id: String) -> Result<(), String> {
    let manager = ConfigManager::new().map_err(|e| e.to_string())?;
    let mut configs = manager.load_configs().map_err(|e| e.to_string())?;
    
    configs.retain(|c| c.id != config_id);
    
    manager.save_configs(&configs).map_err(|e| e.to_string())
}

#[tauri::command]
fn export_config(config: ChannelConfig, file_path: String) -> Result<(), String> {
    let manager = ConfigManager::new().map_err(|e| e.to_string())?;
    manager.export_config(&config, &file_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn import_config(file_path: String) -> Result<ChannelConfig, String> {
    println!("开始导入配置文件: {}", file_path);
    let manager = ConfigManager::new().map_err(|e| {
        let err_msg = format!("创建ConfigManager失败: {}", e);
        println!("{}", err_msg);
        err_msg
    })?;
    
    println!("开始解析配置文件...");
    let config = manager.import_config(&file_path).map_err(|e| {
        let err_msg = format!("解析配置文件失败: {}", e);
        println!("{}", err_msg);
        err_msg
    })?;
    
    println!("配置解析成功，开始保存...");
    // 自动保存导入的配置
    let mut configs = manager.load_configs().map_err(|e| e.to_string())?;
    configs.push(config.clone());
    manager.save_configs(&configs).map_err(|e| e.to_string())?;
    
    println!("配置导入并保存成功");
    Ok(config)
}

// 任务管理命令
#[tauri::command]
fn load_tasks() -> Result<Vec<ReconciliationTask>, String> {
    let manager = TaskManager::new().map_err(|e| e.to_string())?;
    manager.load_tasks().map_err(|e| e.to_string())
}

#[tauri::command]
fn load_task_result(task_id: String) -> Result<ReconciliationResult, String> {
    let manager = TaskManager::new().map_err(|e| e.to_string())?;
    manager.load_task_result(&task_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_task(task_id: String) -> Result<(), String> {
    let manager = TaskManager::new().map_err(|e| e.to_string())?;
    manager.delete_task(&task_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_tasks_by_config(config_id: String) -> Result<Vec<ReconciliationTask>, String> {
    let manager = TaskManager::new().map_err(|e| e.to_string())?;
    manager.get_tasks_by_config(&config_id).map_err(|e| e.to_string())
}

// Double Check 功能：扩大时间范围重新对账
#[tauri::command]
fn double_check_task(
    task_id: String,
    extended_days: usize,
) -> Result<(ReconciliationTask, ReconciliationResult), String> {
    let task_manager = TaskManager::new().map_err(|e| e.to_string())?;
    let original_task = task_manager
        .load_tasks()
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|t| t.task_id == task_id)
        .ok_or_else(|| "任务不存在".to_string())?;

    // 加载配置
    let config_manager = ConfigManager::new().map_err(|e| e.to_string())?;
    let configs = config_manager.load_configs().map_err(|e| e.to_string())?;
    let config = configs
        .iter()
        .find(|c| c.id == original_task.config_id)
        .ok_or_else(|| "配置不存在".to_string())?;

    // 使用历史数据管理器加载更大范围的数据
    let processor = DataProcessor::new().map_err(|e| e.to_string())?;

    // 创建空表
    processor
        .create_empty_tables()
        .map_err(|e| format!("创建表失败: {}", e))?;

    // 从 DuckDB 加载扩大范围的历史数据
    let order_manager = OrderManager::new().map_err(|e| e.to_string())?;
    
    let start_date = chrono::NaiveDate::parse_from_str(&original_task.date_range.start, "%Y-%m-%d")
        .map_err(|e| e.to_string())?
        .checked_sub_signed(chrono::Duration::days(extended_days as i64))
        .ok_or("日期计算失败")?
        .format("%Y-%m-%d")
        .to_string();
    
    let end_date = chrono::NaiveDate::parse_from_str(&original_task.date_range.end, "%Y-%m-%d")
        .map_err(|e| e.to_string())?
        .checked_add_signed(chrono::Duration::days(extended_days as i64))
        .ok_or("日期计算失败")?
        .format("%Y-%m-%d")
        .to_string();

    let source_a_data = order_manager
        .load_historical_data(
            &original_task.config_id,
            &original_task.source_a_name,
            &start_date,
            &end_date,
        )
        .map_err(|e| format!("加载数据源A历史失败: {}", e))?;

    let source_b_data = order_manager
        .load_historical_data(
            &original_task.config_id,
            &original_task.source_b_name,
            &start_date,
            &end_date,
        )
        .map_err(|e| format!("加载数据源B历史失败: {}", e))?;

    // 注意：历史数据已经是清洗后的数据，不需要再次清洗
    processor
        .load_historical_data("source_a", source_a_data)
        .map_err(|e| format!("导入数据源A失败: {}", e))?;

    processor
        .load_historical_data("source_b", source_b_data)
        .map_err(|e| format!("导入数据源B失败: {}", e))?;

    // 验证字段名是否存在
    let source_a_id_field = config.match_config.source_a_id_field.clone();
    let source_b_id_field = config.match_config.source_b_id_field.clone();
    
    // 检查字段是否存在
    let source_a_columns = processor.get_table_columns("source_a")
        .map_err(|e| format!("获取数据源A字段失败: {}", e))?;
    let source_b_columns = processor.get_table_columns("source_b")
        .map_err(|e| format!("获取数据源B字段失败: {}", e))?;
    
    if !source_a_columns.contains(&source_a_id_field) {
        return Err(format!(
            "配置中的字段名 '{}' 在数据源A中不存在。可用字段: {}",
            source_a_id_field,
            source_a_columns.join(", ")
        ));
    }
    
    if !source_b_columns.contains(&source_b_id_field) {
        return Err(format!(
            "配置中的字段名 '{}' 在数据源B中不存在。可用字段: {}",
            source_b_id_field,
            source_b_columns.join(", ")
        ));
    }

    // 查找金额字段（历史数据中已经是清洗后的字段名，如 sourceAAmount）
    let source_a_amount_field = config.source_a_config.mappings
        .iter()
        .find(|m| m.field_type == "OrderAmount")
        .map(|m| m.field_name.clone())
        .unwrap_or_default();

    let source_b_amount_field = config.source_b_config.mappings
        .iter()
        .find(|m| m.field_type == "OrderAmount")
        .map(|m| m.field_name.clone())
        .unwrap_or_default();

    // 验证金额字段是否存在（如果配置了）
    if !source_a_amount_field.is_empty() && !source_a_columns.contains(&source_a_amount_field) {
        return Err(format!(
            "配置中的金额字段名 '{}' 在数据源A中不存在。可用字段: {}",
            source_a_amount_field,
            source_a_columns.join(", ")
        ));
    }
    
    if !source_b_amount_field.is_empty() && !source_b_columns.contains(&source_b_amount_field) {
        return Err(format!(
            "配置中的金额字段名 '{}' 在数据源B中不存在。可用字段: {}",
            source_b_amount_field,
            source_b_columns.join(", ")
        ));
    }

    // 历史数据已经是清洗后的标准格式，状态已经被标准化为 normalized_status
    // 不需要再次进行状态标准化，直接执行对账
    let result = processor
        .perform_reconciliation(
            &source_a_id_field,
            &source_b_id_field,
            &source_a_amount_field,
            &source_b_amount_field,
        )
        .map_err(|e| format!("对账失败: {}", e))?;

    // 创建新任务记录
    let new_task_id = format!("task_{}_doublecheck", chrono::Utc::now().timestamp_millis());
    let created_at = chrono::Utc::now().to_rfc3339();

    let new_task = ReconciliationTask {
        task_id: new_task_id.clone(),
        task_name: format!("{} (Double Check)", original_task.task_name),
        config_id: original_task.config_id.clone(),
        config_name: original_task.config_name.clone(),
        source_a_name: original_task.source_a_name.clone(),
        source_b_name: original_task.source_b_name.clone(),
        task_type: original_task.task_type.clone(),
        date_range: original_task.date_range.clone(),
        created_at,
        source_a_file_name: format!("历史数据(±{}天)", extended_days),
        source_b_file_name: format!("历史数据(±{}天)", extended_days),
        stats: ReconciliationStats {
            matched_count: result.matched.len(),
            only_in_source_a_count: result.only_in_a.len(),
            only_in_source_b_count: result.only_in_b.len(),
            diff_amount_count: result.diff_amount.len(),
            total_source_a: result.matched.len() + result.only_in_a.len() + result.diff_amount.len(),
            total_source_b: result.matched.len() + result.only_in_b.len() + result.diff_amount.len(),
        },
        used_historical_source_a: true,
        used_historical_source_b: true,
    };

    // 保存任务
    task_manager
        .save_task(&new_task, &result)
        .map_err(|e| format!("保存任务失败: {}", e))?;

    Ok((new_task, result))
}

// 首次启动时初始化默认配置
#[tauri::command]
fn init_default_config() -> Result<(), String> {
    let manager = ConfigManager::new().map_err(|e| e.to_string())?;
    let configs = manager.load_configs().map_err(|e| e.to_string())?;

    // 如果已有配置，不创建默认配置
    if !configs.is_empty() {
        return Ok(());
    }

    // 创建默认示例配置
    let default_config = ChannelConfig {
        id: "default-example".to_string(),
        name: "示例配置 - VIDI PAYOUT".to_string(),
        source_a_name: "内部订单系统".to_string(),
        source_b_name: "银行对账单".to_string(),
        config_type: "PAYOUT".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        source_a_config: FileTypeConfig {
            header: 1,
            timezone: "America/Sao_Paulo".to_string(),
            remove_duplicate: true,
            mappings: vec![
                ColumnMapping {
                    id: "1".to_string(),
                    source_column: "transaction_date".to_string(),
                    field_type: "OrderTime".to_string(),
                    field_name: "sourceATime".to_string(),
                    rule_type: "ORDER_TIME_NORMAL".to_string(),
                    rule_config: "".to_string(),
                    save_original: false,
                    format_rules: vec![],
                },
                ColumnMapping {
                    id: "2".to_string(),
                    source_column: "e2e".to_string(),
                    field_type: "OrderString".to_string(),
                    field_name: "sourceAId".to_string(),
                    rule_type: "ORDER_STRING_NORMAL".to_string(),
                    rule_config: "".to_string(),
                    save_original: true,
                    format_rules: vec![FormatRule {
                        rule_type: "pre".to_string(),
                        operation: "DEL_AFTER".to_string(),
                        value: "7".to_string(),
                    }],
                },
                ColumnMapping {
                    id: "3".to_string(),
                    source_column: "status".to_string(),
                    field_type: "OrderStatus".to_string(),
                    field_name: "sourceAStatus".to_string(),
                    rule_type: "ORDER_STATUS_NORMAL".to_string(),
                    rule_config: "".to_string(),
                    save_original: false,
                    format_rules: vec![],
                },
                ColumnMapping {
                    id: "4".to_string(),
                    source_column: "amount".to_string(),
                    field_type: "OrderAmount".to_string(),
                    field_name: "sourceAAmount".to_string(),
                    rule_type: "ORDER_AMOUNT_NORMAL".to_string(),
                    rule_config: "".to_string(),
                    save_original: false,
                    format_rules: vec![],
                },
            ],
        },
        source_b_config: FileTypeConfig {
            header: 1,
            timezone: "America/Sao_Paulo".to_string(),
            remove_duplicate: false,
            mappings: vec![
                ColumnMapping {
                    id: "5".to_string(),
                    source_column: "date".to_string(),
                    field_type: "OrderTime".to_string(),
                    field_name: "sourceBTime".to_string(),
                    rule_type: "ORDER_TIME_NORMAL".to_string(),
                    rule_config: "".to_string(),
                    save_original: false,
                    format_rules: vec![],
                },
                ColumnMapping {
                    id: "6".to_string(),
                    source_column: "e2eId".to_string(),
                    field_type: "OrderString".to_string(),
                    field_name: "sourceBId".to_string(),
                    rule_type: "ORDER_STRING_NORMAL".to_string(),
                    rule_config: "".to_string(),
                    save_original: false,
                    format_rules: vec![],
                },
                ColumnMapping {
                    id: "7".to_string(),
                    source_column: "bank_status".to_string(),
                    field_type: "OrderStatus".to_string(),
                    field_name: "sourceBStatus".to_string(),
                    rule_type: "ORDER_STATUS_NORMAL".to_string(),
                    rule_config: "".to_string(),
                    save_original: false,
                    format_rules: vec![],
                },
                ColumnMapping {
                    id: "8".to_string(),
                    source_column: "transaction_amount".to_string(),
                    field_type: "OrderAmount".to_string(),
                    field_name: "sourceBAmount".to_string(),
                    rule_type: "ORDER_AMOUNT_NORMAL".to_string(),
                    rule_config: "".to_string(),
                    save_original: false,
                    format_rules: vec![],
                },
            ],
        },
        match_config: MatchConfig {
            source_a_id_field: "sourceAId".to_string(),
            source_a_status_mapping: vec![
                StatusMapping {
                    source_status: vec!["PAID".to_string(), "COMPLETED".to_string(), "SUCCESS".to_string()],
                    target_status: "PAID".to_string(),
                },
                StatusMapping {
                    source_status: vec!["REJECTED".to_string(), "FAILED".to_string()],
                    target_status: "REJECTED".to_string(),
                },
            ],
            source_b_id_field: "sourceBId".to_string(),
            source_b_status_mapping: vec![StatusMapping {
                source_status: vec!["PAID".to_string(), "SETTLED".to_string()],
                target_status: "PAID".to_string(),
            }],
            use_historical_source_a: false,
            use_historical_source_b: false,
            history_days: 5,
        },
    };

    let configs = vec![default_config];
    manager.save_configs(&configs).map_err(|e| e.to_string())?;

    Ok(())
}

// 订单管理命令
#[tauri::command]
fn upload_order_file(
    config_id: String,
    config_name: String,
    source_name: String, // 改为 source_name
    file_path: String,
    file_name: String,
    upload_date: String, // 新增：用于去重的日期 YYYY-MM-DD
    header_row: usize,
    mappings: Vec<ColumnMapping>,
) -> Result<OrderFile, String> {
    log::info!("Uploading order file: file_name={}, config_id={}, source_name={}", 
               file_name, config_id, source_name);
    
    let processor = DataProcessor::new()
        .map_err(|e| {
            let err_msg = format!("Failed to create DataProcessor: {}", e);
            log::error!("{}", err_msg);
            err_msg
        })?;
    
    let manager = OrderManager::new()
        .map_err(|e| {
            let err_msg = format!("Failed to create OrderManager: {}", e);
            log::error!("{}", err_msg);
            err_msg
        })?;
    
    // 加载并清洗数据
    let table_name = "temp_data";
    processor
        .load_csv_to_table(&file_path, table_name, header_row)
        .map_err(|e| format!("加载文件失败: {}", e))?;
    
    processor
        .apply_data_cleaning(table_name, &mappings)
        .map_err(|e| format!("清洗数据失败: {}", e))?;
    
    let records = processor
        .get_table_data(table_name)
        .map_err(|e| format!("获取数据失败: {}", e))?;
    
    // 保存到 DuckDB（带去重）
    let order_file = manager
        .save_order_data(
            &config_id,
            &config_name,
            &source_name,
            &file_name,
            &upload_date,
            records,
        )
        .map_err(|e| format!("保存失败: {}", e))?;
    
    Ok(order_file)
}

#[tauri::command]
fn list_order_files() -> Result<Vec<OrderFile>, String> {
    log::info!("Listing order files");
    let manager = OrderManager::new()
        .map_err(|e| {
            let err_msg = format!("Failed to create OrderManager: {}", e);
            log::error!("{}", err_msg);
            err_msg
        })?;
    
    manager.list_order_files()
        .map_err(|e| {
            let err_msg = format!("Failed to list order files: {}", e);
            log::error!("{}", err_msg);
            err_msg
        })
}

#[tauri::command]
fn query_orders(
    config_id: Option<String>,
    source_name: Option<String>,
    conditions: Vec<QueryCondition>,
    limit: Option<usize>,
) -> Result<Vec<std::collections::HashMap<String, serde_json::Value>>, String> {
    log::info!("Querying orders: config_id={:?}, source_name={:?}, conditions_count={}, limit={:?}",
               config_id, source_name, conditions.len(), limit);
    
    let manager = OrderManager::new()
        .map_err(|e| {
            let err_msg = format!("Failed to create OrderManager: {}", e);
            log::error!("{}", err_msg);
            err_msg
        })?;
    
    manager
        .query_orders(
            config_id.as_deref(),
            source_name.as_deref(),
            conditions,
            limit,
        )
        .map_err(|e| {
            let err_msg = format!("Failed to query orders: {}", e);
            log::error!("{}", err_msg);
            err_msg
        })
}

#[tauri::command]
fn delete_order_file(file_id: String) -> Result<(), String> {
    let manager = OrderManager::new().map_err(|e| e.to_string())?;
    manager.delete_order_file(&file_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_order_files_by_config(config_id: String) -> Result<Vec<OrderFile>, String> {
    let manager = OrderManager::new().map_err(|e| e.to_string())?;
    manager.get_files_by_config(&config_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn cleanup_orders_before_date(before_date: String) -> Result<usize, String> {
    let manager = OrderManager::new().map_err(|e| e.to_string())?;
    manager.cleanup_orders_before_date(&before_date).map_err(|e| e.to_string())
}

/// 清除所有订单数据
#[tauri::command]
fn clear_all_orders() -> Result<usize, String> {
    let manager = OrderManager::new().map_err(|e| e.to_string())?;
    manager.clear_all_orders().map_err(|e| e.to_string())
}

/// 清除所有历史数据（任务、订单、配置）
#[tauri::command]
fn reset_all_data() -> Result<(), String> {
    let data_dir = dirs::data_dir()
        .ok_or("无法获取数据目录")?
        .join(".file-compare");
    
    if data_dir.exists() {
        std::fs::remove_dir_all(&data_dir).map_err(|e| e.to_string())?;
        println!("已清除所有历史数据");
    }
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志记录器
    logger::init_logger();
    log::info!("Application starting...");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            read_csv_headers,
            reconcile,
            export_results,
            load_configs,
            save_config,
            delete_config,
            export_config,
            import_config,
            load_tasks,
            load_task_result,
            delete_task,
            get_tasks_by_config,
            double_check_task,
            init_default_config,
            upload_order_file,
            list_order_files,
            query_orders,
            delete_order_file,
            get_order_files_by_config,
            cleanup_orders_before_date,
            clear_all_orders,
            reset_all_data
        ])
        .setup(|_app| {
            log::info!("Tauri application setup completed");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
