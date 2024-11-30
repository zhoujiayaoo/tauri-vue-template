use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub server_ip: String,
    pub server_username: String,
    pub server_password: String,
    pub project_path: String,
}

// 读取文件内容
pub fn get_config_file_content() -> String {
    let config_path = get_config_path();

    // Try to open the config file
    match File::open(&config_path) {
        Ok(mut file) => {
            // Read the content of the config file
            let mut contents = String::new();
            if let Ok(_) = file.read_to_string(&mut contents) {
                return contents;
            }
        }
        Err(_) => {
            // If the file doesn't exist or an error occurred, return an empty string
            let _ = File::create(&config_path); // Ignore errors when creating the file
            return String::new();
        }
    }

    String::new()
}

// 将配置字符串保存到文件中
pub fn save_config_to_file(config_data: &str) -> io::Result<()> {
    let config_path: PathBuf = get_config_path();

    // Try to open the config file
    let mut file = File::create(&config_path)?;
    file.write_all(config_data.as_bytes())?;

    Ok(())
}

fn get_config_path() -> PathBuf {
    // 读取配置文件
    let mut path = get_executable_directory(); // 软件根目录
    path.push("config.json");
    path
}

pub fn get_executable_directory() -> PathBuf {
    // Check if the application is running in development mode
    let is_development = cfg!(debug_assertions);

    if is_development {
        // Use the fixed path during development
        PathBuf::from("G:\\poodle")
    } else {
        // Switch to the executable directory after packaging
        let mut path = env::current_exe().expect("Failed to get executable path");
        path.pop(); // Remove executable name, leaving only the directory
        path
    }
}


pub fn get_class_full_path(class_name: &str) -> String {
    let path = get_executable_directory().join("data").join(class_name);
    path.to_str().expect("路径不是有效的 UTF-8 字符串").to_string()
}