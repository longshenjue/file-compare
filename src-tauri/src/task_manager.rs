use crate::models::*;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct TaskManager {
    tasks_dir: PathBuf,
    tasks_file: PathBuf,
}

impl TaskManager {
    pub fn new() -> Result<Self> {
        let tasks_dir = dirs::data_dir()
            .context("无法获取数据目录")?
            .join(".file-compare")
            .join("tasks");

        fs::create_dir_all(&tasks_dir)?;

        let tasks_file = tasks_dir.join("tasks.json");

        Ok(Self {
            tasks_dir: tasks_dir.clone(),
            tasks_file,
        })
    }

    /// 加载所有任务
    pub fn load_tasks(&self) -> Result<Vec<ReconciliationTask>> {
        if !self.tasks_file.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.tasks_file)?;
        
        // 直接使用最新格式解析
        match serde_json::from_str::<Vec<ReconciliationTask>>(&content) {
            Ok(tasks) => Ok(tasks),
            Err(e) => {
                // 如果解析失败，记录错误并删除损坏的文件
                eprintln!("任务数据格式不正确，将重置: {}", e);
                fs::remove_file(&self.tasks_file)?;
                Ok(Vec::new())
            }
        }
    }

    /// 保存所有任务
    pub fn save_tasks(&self, tasks: &[ReconciliationTask]) -> Result<()> {
        let json = serde_json::to_string_pretty(tasks)?;
        fs::write(&self.tasks_file, json)?;
        Ok(())
    }

    /// 保存单个任务和详细结果
    pub fn save_task(
        &self,
        task: &ReconciliationTask,
        result: &ReconciliationResult,
    ) -> Result<()> {
        // 保存任务元数据
        let mut tasks = self.load_tasks()?;

        // 查找是否存在相同ID的任务
        if let Some(index) = tasks.iter().position(|t| t.task_id == task.task_id) {
            tasks[index] = task.clone();
        } else {
            tasks.push(task.clone());
        }

        self.save_tasks(&tasks)?;

        // 保存详细结果到独立文件
        let result_file = self.tasks_dir.join(format!("{}_result.json", task.task_id));
        let result_json = serde_json::to_string_pretty(result)?;
        fs::write(result_file, result_json)?;

        Ok(())
    }

    /// 加载任务的详细结果
    pub fn load_task_result(&self, task_id: &str) -> Result<ReconciliationResult> {
        let result_file = self.tasks_dir.join(format!("{}_result.json", task_id));

        if !result_file.exists() {
            anyhow::bail!("任务结果不存在");
        }

        let content = fs::read_to_string(result_file)?;
        let result: ReconciliationResult = serde_json::from_str(&content)?;

        Ok(result)
    }

    /// 删除任务
    pub fn delete_task(&self, task_id: &str) -> Result<()> {
        let mut tasks = self.load_tasks()?;
        tasks.retain(|t| t.task_id != task_id);
        self.save_tasks(&tasks)?;

        // 删除详细结果文件
        let result_file = self.tasks_dir.join(format!("{}_result.json", task_id));
        if result_file.exists() {
            fs::remove_file(result_file)?;
        }

        Ok(())
    }

    /// 获取指定配置的任务列表
    pub fn get_tasks_by_config(&self, config_id: &str) -> Result<Vec<ReconciliationTask>> {
        let tasks = self.load_tasks()?;
        Ok(tasks.into_iter().filter(|t| t.config_id == config_id).collect())
    }
}

