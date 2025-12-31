use crate::models::*;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

const CONFIG_DIR_NAME: &str = "reconciliation_configs";
const CONFIG_FILE_NAME: &str = "configs.json";

pub struct ConfigManager {
    config_dir: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config_dir = Self::get_config_dir()?;
        
        // 确保配置目录存在
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .context("无法创建配置目录")?;
        }
        
        Ok(Self { config_dir })
    }
    
    fn get_config_dir() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .context("无法获取用户主目录")?;
        Ok(home_dir.join(".file-compare").join(CONFIG_DIR_NAME))
    }
    
    fn get_config_file_path(&self) -> PathBuf {
        self.config_dir.join(CONFIG_FILE_NAME)
    }
    
    pub fn load_configs(&self) -> Result<Vec<ChannelConfig>> {
        let file_path = self.get_config_file_path();
        
        if !file_path.exists() {
            return Ok(Vec::new());
        }
        
        let content = fs::read_to_string(&file_path)
            .context("无法读取配置文件")?;
        
        let configs: Vec<ChannelConfig> = serde_json::from_str(&content)
            .context("配置文件格式错误")?;
        
        Ok(configs)
    }
    
    pub fn save_configs(&self, configs: &[ChannelConfig]) -> Result<()> {
        let file_path = self.get_config_file_path();
        let content = serde_json::to_string_pretty(configs)
            .context("无法序列化配置")?;
        
        fs::write(&file_path, content)
            .context("无法写入配置文件")?;
        
        Ok(())
    }
    
    pub fn export_config(&self, config: &ChannelConfig, export_path: &str) -> Result<()> {
        let content = serde_json::to_string_pretty(config)
            .context("无法序列化配置")?;
        
        fs::write(export_path, content)
            .context("无法导出配置文件")?;
        
        Ok(())
    }
    
    pub fn import_config(&self, import_path: &str) -> Result<ChannelConfig> {
        let content = fs::read_to_string(import_path)
            .context("无法读取导入文件")?;
        
        // 尝试解析，提供详细的错误信息
        let mut config: ChannelConfig = serde_json::from_str(&content)
            .map_err(|e| {
                anyhow::anyhow!(
                    "配置文件格式错误: {}\n\n请检查：\n1. 是否包含 sourceAName 和 sourceBName 字段\n2. 是否包含 sourceAConfig 和 sourceBConfig 字段\n3. matchConfig 中是否包含 sourceAIdField 和 sourceBIdField\n4. 所有字段名是否使用 camelCase 格式",
                    e
                )
            })?;
        
        // 生成新的ID和时间戳
        config.id = format!("config-{}", chrono::Utc::now().timestamp_millis());
        config.created_at = chrono::Utc::now().to_rfc3339();
        config.updated_at = chrono::Utc::now().to_rfc3339();
        
        Ok(config)
    }
}

