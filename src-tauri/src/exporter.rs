use crate::models::ReconciliationResult;
use anyhow::Result;
use std::collections::HashMap;
use std::fs::File;
use csv::Writer;

pub fn export_to_csv(
    results: &ReconciliationResult,
    export_type: &str,
    file_path: &str,
) -> Result<()> {
    match export_type {
        "matched" => write_csv(&results.matched, file_path)?,
        "onlyInA" => write_csv(&results.only_in_a, file_path)?,
        "onlyInB" => write_csv(&results.only_in_b, file_path)?,
        "diffAmount" => write_csv_diff_amount(&results.diff_amount, file_path)?,
        "all" => {
            return export_all_to_csv(results, file_path);
        }
        _ => write_csv(&results.matched, file_path)?,
    };

    Ok(())
}

fn export_all_to_csv(results: &ReconciliationResult, base_path: &str) -> Result<()> {
    // Remove .csv extension if present
    let base_path = base_path.trim_end_matches(".csv");
    
    // Export each category with descriptive names
    write_csv(&results.matched, &format!("{}_完全匹配.csv", base_path))?;
    write_csv(&results.only_in_a, &format!("{}_仅在数据源A.csv", base_path))?;
    write_csv(&results.only_in_b, &format!("{}_仅在数据源B.csv", base_path))?;
    write_csv_diff_amount(&results.diff_amount, &format!("{}_金额差异.csv", base_path))?;
    
    Ok(())
}

fn write_csv(data: &[HashMap<String, serde_json::Value>], file_path: &str) -> Result<()> {
    if data.is_empty() {
        // Create empty file
        File::create(file_path)?;
        return Ok(());
    }

    let file = File::create(file_path)?;
    let mut writer = Writer::from_writer(file);

    // 获取所有唯一的列名
    let mut all_headers: Vec<String> = Vec::new();
    for row in data {
        for key in row.keys() {
            if !all_headers.contains(key) {
                all_headers.push(key.clone());
            }
        }
    }

    // 智能排序：按照字段重要性排序
    let mut headers: Vec<String> = Vec::new();
    
    // 1. 首先添加ID字段（sourceAId, sourceBId等）
    for header in &all_headers {
        if header.to_lowercase().contains("id") && !header.contains("normalized") {
            headers.push(header.clone());
        }
    }
    
    // 2. 添加时间字段
    for header in &all_headers {
        if header.to_lowercase().contains("time") && !headers.contains(header) {
            headers.push(header.clone());
        }
    }
    
    // 3. 添加金额字段
    for header in &all_headers {
        if header.to_lowercase().contains("amount") && !headers.contains(header) {
            headers.push(header.clone());
        }
    }
    
    // 4. 添加状态字段
    for header in &all_headers {
        if header.to_lowercase().contains("status") && !headers.contains(header) {
            headers.push(header.clone());
        }
    }
    
    // 5. 添加其他标准化字段
    for header in &all_headers {
        if !headers.contains(header) && (
            header.starts_with("source") || 
            header.contains("normalized") ||
            header.contains("original")
        ) {
            headers.push(header.clone());
        }
    }

    // 写入表头
    writer.write_record(&headers)?;

    // 写入数据
    for row in data {
        let record: Vec<String> = headers
            .iter()
            .map(|h| {
                row.get(h)
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        serde_json::Value::Null => String::new(),
                        _ => v.to_string(),
                    })
                    .unwrap_or_default()
            })
            .collect();
        writer.write_record(&record)?;
    }

    writer.flush()?;
    Ok(())
}

/// 专门处理金额差异的导出，清晰显示两边的金额
fn write_csv_diff_amount(data: &[HashMap<String, serde_json::Value>], file_path: &str) -> Result<()> {
    if data.is_empty() {
        // Create empty file
        File::create(file_path)?;
        return Ok(());
    }

    let file = File::create(file_path)?;
    let mut writer = Writer::from_writer(file);

    // 获取所有唯一的列名
    let mut all_headers: Vec<String> = Vec::new();
    for row in data {
        for key in row.keys() {
            if !all_headers.contains(key) {
                all_headers.push(key.clone());
            }
        }
    }

    // 智能排序：按照字段重要性排序，特别突出金额差异
    let mut headers: Vec<String> = Vec::new();
    
    // 1. 添加ID字段
    for header in &all_headers {
        if header.to_lowercase().contains("id") && !header.contains("normalized") {
            headers.push(header.clone());
        }
    }
    
    // 2. 添加时间字段
    for header in &all_headers {
        if header.to_lowercase().contains("time") && !headers.contains(header) {
            headers.push(header.clone());
        }
    }
    
    // 3. 特别处理金额字段：分别显示sourceA和sourceB的金额
    let mut source_a_amount: Option<String> = None;
    let mut source_b_amount: Option<String> = None;
    
    for header in &all_headers {
        if header.to_lowercase().contains("amount") {
            if header.to_lowercase().contains("sourcea") {
                source_a_amount = Some(header.clone());
            } else if header.to_lowercase().contains("sourceb") {
                source_b_amount = Some(header.clone());
            }
        }
    }
    
    // 按照 sourceA -> sourceB 的顺序添加金额字段
    if let Some(amount) = source_a_amount.clone() {
        headers.push(amount);
    }
    if let Some(amount) = source_b_amount.clone() {
        headers.push(amount);
    }
    
    // 添加金额差异字段（如果存在的话，我们可以计算）
    let has_amount_diff = source_a_amount.is_some() && source_b_amount.is_some();
    
    // 4. 添加状态字段
    for header in &all_headers {
        if header.to_lowercase().contains("status") && !headers.contains(header) {
            headers.push(header.clone());
        }
    }
    
    // 5. 添加其他标准化字段
    for header in &all_headers {
        if !headers.contains(header) && (
            header.starts_with("source") || 
            header.contains("normalized") ||
            header.contains("original")
        ) {
            headers.push(header.clone());
        }
    }

    // 如果需要计算金额差异，添加差异列
    if has_amount_diff {
        headers.push("金额差异".to_string());
    }

    // 写入表头
    writer.write_record(&headers)?;

    // 写入数据
    for row in data {
        let mut record: Vec<String> = headers
            .iter()
            .filter(|h| *h != "金额差异")  // 先过滤掉金额差异字段
            .map(|h| {
                row.get(h)
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::Bool(b) => b.to_string(),
                        serde_json::Value::Null => String::new(),
                        _ => v.to_string(),
                    })
                    .unwrap_or_default()
            })
            .collect();
        
        // 计算并添加金额差异
        if has_amount_diff {
            if let (Some(a_field), Some(b_field)) = (&source_a_amount, &source_b_amount) {
                let amount_a = row.get(a_field)
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let amount_b = row.get(b_field)
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let diff = amount_a - amount_b;
                record.push(format!("{:.2}", diff));
            }
        }
        
        writer.write_record(&record)?;
    }

    writer.flush()?;
    Ok(())
}

