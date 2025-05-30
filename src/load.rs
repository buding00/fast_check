use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn load_path(path_str: &str) -> Result<Vec<PathBuf>, io::Error> {
    // 1. 解析初始路径：如果是 "./"，则使用当前目录，否则使用给定路径字符串
    let initial_path_buf = if path_str == "./" {
        env::current_dir()?.to_path_buf()
    } else {
        PathBuf::from(path_str)
    };

    // 2. 获取初始路径的规范化、绝对路径。
    // fs::canonicalize 会解析符号链接、".." 和 "."，并检查路径是否存在。
    let canonical_initial_path = match fs::canonicalize(&initial_path_buf) {
        Ok(p) => p,
        Err(e) => {
            return Err(io::Error::new(
                e.kind(),
                format!("无法规范化路径 '{}': {}", initial_path_buf.display(), e),
            ));
        }
    };

    let mut files_found: Vec<PathBuf> = Vec::new();

    // 3. 根据路径是文件还是目录进行处理
    if canonical_initial_path.is_file() {
        // 如果是文件，将其绝对路径添加到列表中
        files_found.push(canonical_initial_path);
    } else if canonical_initial_path.is_dir() {
        // 如果是目录，递归收集其中的所有文件
        collect_files_recursively(&canonical_initial_path, &mut files_found)?;
    }
    Ok(files_found)
}

// 辅助函数，用于递归地收集目录中的文件路径
fn collect_files_recursively(dir_path: &Path, files_list: &mut Vec<PathBuf>) -> io::Result<()> {
    // 确保我们只尝试读取实际目录的内容
    if !dir_path.is_dir() {
        return Ok(());
    }

    for entry_result in fs::read_dir(dir_path)? {
        let entry = entry_result?;
        let path = entry.path();

        if path.is_dir() {
            // 如果是目录，则递归调用
            collect_files_recursively(&path, files_list)?;
        } else if path.is_file() {
            // 如果是文件，获取其规范化的绝对路径
            match fs::canonicalize(&path) {
                Ok(canonical_path) => {
                    files_list.push(canonical_path);
                }
                Err(e) => {
                    // 如果特定文件无法规范化（例如，损坏的符号链接），
                    // 此处选择传播错误。或者，可以记录错误并跳过该文件。
                    // eprintln!("警告：无法规范化路径 {:?}：{}", path, e);
                    return Err(e);
                }
            }
        }
    }
    Ok(())
}

mod test {
    #[test]
    pub fn test_load() {
        use super::*;
        let path = "./";
        match load_path(path) {
            Ok(files) => {
                for file in files {
                    println!("找到文件: {:?}", file); // 打印文件路径
                    match fs::read(&file) {
                        // 读取文件内容
                        Ok(contents) => {
                            println!("文件 {:?} 的内容长度: {}", file, contents.len()); // 打印文件内容长度
                        }
                        Err(e) => {
                            eprintln!("读取文件 {:?} 内容时出错: {}", file, e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("加载路径时出错: {}", e);
            }
        }
    }
}
