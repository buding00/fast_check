mod cli;
mod load;
mod rule;

use rust_embed::Embed;

use std::{path::PathBuf, sync::Arc};
// 移除了未使用的 mpsc
use tokio::sync::{Mutex, mpsc}; // 添加了 mpsc 和 Mutex

#[derive(Embed)]
#[folder = "rules/"]
#[include = "*.yar"]
struct Asset;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = cli::cli().get_matches();
    match matches.subcommand() {
        Some(("dp", sub_matches)) => {
            let default_path_str = "./".to_string();
            let path_str = sub_matches
                .get_one::<String>("path")
                .unwrap_or(&default_path_str);
            let default_thread_str = "10".to_string();
            let thread_val_str: String = sub_matches
                .get_one::<String>("thread")
                .cloned() // 从 Option<&String>
                .unwrap_or_else(|| default_thread_str); //
            println!(
                "将使用: {} 线程，开始扫描目录: {}",
                thread_val_str, path_str
            );
            let compiled_rules = rule::new_rule().build();
            let shared_rules = Arc::new(compiled_rules);
            let parsed_num_workers: usize = thread_val_str.parse().unwrap_or_else(|e| {
                eprintln!(
                    "无法将线程数 '{}' 解析为数字: {}. 使用默认值 10.",
                    thread_val_str, e
                );
                10
            });
            let num_workers = std::cmp::max(1, parsed_num_workers);

            let (job_tx, job_rx) = mpsc::channel::<(PathBuf, Vec<u8>)>(1000);
            let shared_job_rx = Arc::new(Mutex::new(job_rx));
            let mut worker_handles = Vec::new();
            println!("启动 {} 个扫描工作者...", num_workers);
            for _ in 0..num_workers {
                let worker_rules_ref = Arc::clone(&shared_rules);
                let worker_job_rx_ref = Arc::clone(&shared_job_rx);
                let handle = tokio::spawn(async move {
                    loop {
                        let maybe_job = {
                            let mut rx = worker_job_rx_ref.lock().await;
                            rx.recv().await
                        };
                        match maybe_job {
                            Some((file_path, contents)) => {
                                let mut scanner = yara_x::Scanner::new(&worker_rules_ref);
                                match scanner.scan(&contents) {
                                    Ok(matches) => {
                                        if matches.matching_rules().len() > 0 {
                                            println!("文件: {} 匹配以下规则:", file_path.display());
                                            println!();
                                            for i in matches.matching_rules() {
                                                println!("          *   {}", i.identifier());
                                            }
                                        }
                                    }
                                    Err(_) => continue,
                                }
                            }
                            None => {
                                break;
                            }
                        }
                    }
                });
                worker_handles.push(handle);
            }
            match load::load_path(path_str) {
                Ok(files) => {
                    if files.is_empty() {
                        println!("在路径 '{}' 下没有找到符合条件的文件。", path_str);
                    } else {
                        println!("开始异步读取和分发 {} 个文件...", files.len());
                        for file_path_buf in files {
                            match tokio::fs::read(&file_path_buf).await {
                                Ok(contents) => {
                                    if job_tx
                                        .send((file_path_buf.clone(), contents))
                                        .await
                                        .is_err()
                                    {
                                        eprintln!(
                                            "无法发送作业到处理队列 (文件: {:?})，可能所有工作者都已退出。",
                                            file_path_buf
                                        );
                                        break;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("读取文件 {:?} 时出错: {}", file_path_buf, e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("加载路径 '{}' 时出错: {}", path_str, e);
                }
            }
            println!("所有文件已尝试分发。正在关闭作业通道...");
            drop(job_tx);
            for (i, handle) in worker_handles.into_iter().enumerate() {
                if let Err(e) = handle.await {
                    eprintln!("工作者任务 {} 执行时发生错误: {}", i, e);
                }
            }
            println!("所有扫描任务已完成。");
        }
        _ => {}
    }
    Ok(())
}
