// 渠道配置（持久化保存）
export interface ChannelConfig {
  id: string;
  name: string; // 配置名称
  sourceAName: string; // 数据源A的自定义名称（如 "内部订单系统"、"银行A"）
  sourceBName: string; // 数据源B的自定义名称（如 "银行对账单"、"银行B"）
  type: string; // PAYOUT/PAYIN
  createdAt: string;
  updatedAt: string;
  
  // 数据源A配置
  sourceAConfig: {
    header: number;
    timezone: string;
    removeDuplicate: boolean;
    mappings: ColumnMapping[];
  };
  
  // 数据源B配置
  sourceBConfig: {
    header: number;
    timezone: string;
    removeDuplicate: boolean;
    mappings: ColumnMapping[];
  };
  
  // 匹配配置
  matchConfig: {
    sourceAIdField: string;
    sourceAStatusMapping: StatusMapping[];
    sourceBIdField: string;
    sourceBStatusMapping: StatusMapping[];
    useHistoricalSourceA?: boolean;
    useHistoricalSourceB?: boolean;
    historyDays?: number;
  };
}

// 文件上传配置（临时使用）
export interface FileConfig {
  sourceName: string; // 数据源名称
  type: string;
  dateRange: {
    start: string;
    end: string;
  };
  filePath: string;
  fileName: string;
  header: number;
  timezone: string;
  removeDuplicate: boolean;
}

export interface ColumnMapping {
  id: string;
  sourceColumn: string;
  fieldType: 'OrderTime' | 'OrderStatus' | 'OrderString' | 'OrderAmount';
  fieldName: string;
  ruleType: string;
  ruleConfig: string;
  saveOriginal: boolean;
  formatRules: FormatRule[];
}

export interface FormatRule {
  type: 'pre' | 'post';
  operation: string;
  value: string;
}

export interface StatusMapping {
  sourceStatus: string[];
  targetStatus: string;
}

export interface MatchConfig {
  sourceAIdField: string;
  sourceAStatusMapping: StatusMapping[];
  sourceBIdField: string;
  sourceBStatusMapping: StatusMapping[];
  useHistoricalSourceA?: boolean;
  useHistoricalSourceB?: boolean;
  historyDays?: number;
}

export interface ReconciliationResult {
  matched: any[];
  onlyInA: any[];
  onlyInB: any[];
  diffAmount: any[];
}

export interface ReconciliationStats {
  matchedCount: number;
  onlyInSourceACount: number;
  onlyInSourceBCount: number;
  diffAmountCount: number;
  totalSourceA: number;
  totalSourceB: number;
}

export interface ReconciliationTask {
  taskId: string;
  taskName: string;
  configId: string;
  configName: string;
  sourceAName: string;
  sourceBName: string;
  taskType: string;
  dateRange: {
    start: string;
    end: string;
  };
  createdAt: string;
  sourceAFileName: string;
  sourceBFileName: string;
  stats: ReconciliationStats;
  usedHistoricalSourceA: boolean;
  usedHistoricalSourceB: boolean;
}

export interface OrderFile {
  fileId: string;
  fileName: string;
  configId: string;
  configName: string;
  sourceName: string; // 数据源名称
  uploadTime: string;
  recordCount: number;
  dateRange?: {
    start: string;
    end: string;
  };
}

export interface HistoryConfig {
  useHistoricalSourceA: boolean;
  useHistoricalSourceB: boolean;
  historicalDataRange: number;
}

export interface OrderQueryCondition {
  field: string;
  operator: 'equals' | 'contains' | 'gt' | 'lt' | 'between';
  value: string;
  value2?: string;
}

export const RULE_TYPES = {
  ORDER_TIME_NORMAL: 'ORDER_TIME_NORMAL',
  ORDER_TIME_TIMESTAMP: 'ORDER_TIME_TIMESTAMP',
  ORDER_STATUS_NORMAL: 'ORDER_STATUS_NORMAL',
  ORDER_STRING_NORMAL: 'ORDER_STRING_NORMAL',
  ORDER_AMOUNT_NORMAL: 'ORDER_AMOUNT_NORMAL',
};

export const FORMAT_OPERATIONS = [
  { value: 'DEL_PRE', label: 'DEL_PRE' },
  { value: 'DEL_AFTER', label: 'DEL_AFTER' },
  { value: 'DEL_CHAR', label: 'DEL_CHAR' },
  { value: 'REPLACE_TWO_CHAR', label: 'REPLACE_TWO_CHAR' },
  { value: 'BRA_VALUE', label: 'BRA_VALUE' },
  { value: 'DIVIDE_NUMBER', label: 'DIVIDE_NUMBER' },
  { value: 'ABS_VALUE', label: 'ABS_VALUE' },
  { value: 'ADD_CHAR_PRE', label: 'ADD_CHAR_PRE' },
  { value: 'ADD_CHAR_AFTER', label: 'ADD_CHAR_AFTER' },
  { value: 'XENDIT_TIME', label: 'XENDIT_TIME' },
];

export const TIMEZONES = [
  'America/Sao_Paulo',
  'America/New_York',
  'Europe/London',
  'Asia/Shanghai',
  'Asia/Tokyo',
  'UTC',
];

import { invoke } from '@tauri-apps/api/core';

// 工具函数：保存配置（调用 Tauri 命令）
export async function saveConfigToStorage(config: ChannelConfig): Promise<void> {
  await invoke('save_config', { config });
}

// 工具函数：加载所有配置（调用 Tauri 命令）
export async function loadConfigsFromStorage(): Promise<ChannelConfig[]> {
  try {
    const configs = await invoke<ChannelConfig[]>('load_configs');
    return configs;
  } catch (error) {
    console.error('加载配置失败:', error);
    return [];
  }
}

// 工具函数：删除配置（调用 Tauri 命令）
export async function deleteConfigFromStorage(id: string): Promise<void> {
  await invoke('delete_config', { configId: id });
}

// 工具函数：获取单个配置
export async function getConfigById(id: string): Promise<ChannelConfig | null> {
  const configs = await loadConfigsFromStorage();
  return configs.find(c => c.id === id) || null;
}

// 工具函数：导出配置
export async function exportConfig(config: ChannelConfig, filePath: string): Promise<void> {
  await invoke('export_config', { config, filePath });
}

// 工具函数：导入配置
export async function importConfig(filePath: string): Promise<ChannelConfig> {
  return await invoke<ChannelConfig>('import_config', { filePath });
}
