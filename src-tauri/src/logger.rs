use log::LevelFilter;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

static LOG_FILE: Mutex<Option<File>> = Mutex::new(None);

pub fn init_logger() {
    // 设置 panic hook 来捕获崩溃信息
    std::panic::set_hook(Box::new(|panic_info| {
        let message = format!(
            "PANIC: {}\nLocation: {:?}\nBacktrace:\n{}",
            panic_info.payload().downcast_ref::<&str>()
                .map(|s| *s)
                .or_else(|| panic_info.payload().downcast_ref::<String>().map(|s| s.as_str()))
                .unwrap_or("Unknown panic"),
            panic_info.location(),
            std::backtrace::Backtrace::capture()
        );
        
        log::error!("{}", message);
        eprintln!("{}", message);
    }));

    // 获取日志文件路径
    let log_file_path = get_log_file_path();
    
    // 创建日志目录
    if let Some(parent) = log_file_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("Failed to create log directory: {}", e);
        }
    }

    // 打开日志文件
    match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
    {
        Ok(file) => {
            *LOG_FILE.lock().unwrap() = Some(file);
        }
        Err(e) => {
            eprintln!("Failed to open log file at {:?}: {}", log_file_path, e);
        }
    }

    // 初始化 env_logger，同时写入文件和控制台
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Info)
        .format(|buf, record| {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let level = record.level();
            let target = record.target();
            let args = record.args();
            
            let log_line = format!(
                "[{}] {} [{}] {}\n",
                timestamp, level, target, args
            );

            // 写入控制台
            writeln!(buf, "{}", log_line.trim())?;

            // 写入文件
            if let Some(ref mut file) = *LOG_FILE.lock().unwrap() {
                let _ = file.write_all(log_line.as_bytes());
                let _ = file.flush();
            }

            Ok(())
        })
        .init();

    log::info!("Logger initialized. Log file: {:?}", log_file_path);
}

fn get_log_file_path() -> PathBuf {
    // 尝试获取应用数据目录
    if let Some(data_dir) = dirs::data_dir() {
        let log_dir = data_dir.join(".file-compare").join("logs");
        return log_dir.join(format!("app_{}.log", chrono::Local::now().format("%Y%m%d")));
    }

    // 如果无法获取数据目录，使用当前目录
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("logs")
        .join(format!("app_{}.log", chrono::Local::now().format("%Y%m%d")))
}

