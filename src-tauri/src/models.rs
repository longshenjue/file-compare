use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// 渠道配置（持久化保存）
// 改为通用的双数据源模型，不再限定为 orders/bank
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelConfig {
    pub id: String,
    pub name: String,
    // 数据源 A 的自定义名称（如 "订单系统"、"银行A"、"Paypal"）
    pub source_a_name: String,
    // 数据源 B 的自定义名称（如 "银行对账单"、"银行B"、"Stripe"）
    pub source_b_name: String,
    #[serde(rename = "type")]
    pub config_type: String,
    pub created_at: String,
    pub updated_at: String,
    pub source_a_config: FileTypeConfig,
    pub source_b_config: FileTypeConfig,
    pub match_config: MatchConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileTypeConfig {
    pub header: usize,
    pub timezone: String,
    pub remove_duplicate: bool,
    pub mappings: Vec<ColumnMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileConfig {
    pub source_name: String, // 数据源名称（对应 source_a_name 或 source_b_name）
    #[serde(rename = "type")]
    pub file_type: String,
    pub date_range: DateRange,
    pub file_path: String,
    pub file_name: String,
    pub header: usize,
    pub timezone: String,
    pub remove_duplicate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DateRange {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnMapping {
    pub id: String,
    pub source_column: String,
    pub field_type: String,
    pub field_name: String,
    pub rule_type: String,
    pub rule_config: String,
    pub save_original: bool,
    pub format_rules: Vec<FormatRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatRule {
    #[serde(rename = "type")]
    pub rule_type: String,
    pub operation: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusMapping {
    pub source_status: Vec<String>,
    pub target_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchConfig {
    pub source_a_id_field: String,
    pub source_a_status_mapping: Vec<StatusMapping>,
    pub source_b_id_field: String,
    pub source_b_status_mapping: Vec<StatusMapping>,
    // 是否启用历史数据对账（数据源A）
    #[serde(default)]
    pub use_historical_source_a: bool,
    // 是否启用历史数据对账（数据源B）
    #[serde(default)]
    pub use_historical_source_b: bool,
    // 历史数据范围（天数）
    #[serde(default = "default_history_days")]
    pub history_days: usize,
}

fn default_history_days() -> usize {
    5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationResult {
    pub matched: Vec<HashMap<String, serde_json::Value>>,
    pub only_in_a: Vec<HashMap<String, serde_json::Value>>,
    pub only_in_b: Vec<HashMap<String, serde_json::Value>>,
    pub diff_amount: Vec<HashMap<String, serde_json::Value>>,
}

// 对账任务记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationTask {
    pub task_id: String,
    pub task_name: String,
    pub config_id: String,
    pub config_name: String,
    pub source_a_name: String,
    pub source_b_name: String,
    pub task_type: String, // PAYOUT/PAYIN
    pub date_range: DateRange,
    pub created_at: String,
    pub source_a_file_name: String,
    pub source_b_file_name: String,
    // 对账统计
    pub stats: ReconciliationStats,
    // 是否使用了历史数据
    pub used_historical_source_a: bool,
    pub used_historical_source_b: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationStats {
    pub matched_count: usize,
    pub only_in_source_a_count: usize,
    pub only_in_source_b_count: usize,
    pub diff_amount_count: usize,
    pub total_source_a: usize,
    pub total_source_b: usize,
}

