#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


mod confit_utils;
mod jar_utils;
mod ssh_utils;
mod common_service;

use serde_json::Number;
use tauri_plugin_shell::ShellExt;
// use tauri::{CustomMenuItem, Manager, Menu, Submenu};
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder}, Manager,
};

use ssh2::Session;
use std::fs::File;
use std::net::TcpStream;
use std::path::Path;
use std::io::{self, Write, Read, Cursor};
use zip::read::ZipArchive;

use confit_utils::{Config, get_config_file_content, save_config_to_file, get_executable_directory};
use jar_utils::{ClassInfoDTO, extract_class_files_from_directory, process_jar_archive};
use ssh_utils::SshUtils;
use common_service::parse_jps_str;


#[tauri::command]
fn edit_save_handle(config: &str) -> Result<String, tauri::Error> {
    match save_config_to_file(config) {
        Ok(_) => Ok("success".to_string()),
        Err(e) => {
            // Print or log the error for debugging
            eprintln!("Error saving config: {}", e);
            Err(tauri::Error::from(e))
        }
    }
}


//
#[tauri::command]
fn get_java_process_list_handle() -> Result<String, tauri::Error>{
    let config_content = get_config_file_content();
    let config: Config = serde_json::from_str(&config_content)
        .map_err(|e| tauri::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    // ssh
    let ssh = SshUtils::new(&config.server_ip, &config.server_username, &config.server_password)
        .map_err(|e| tauri::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    let java_process_info = ssh.exec_command("jps")
        .map_err(|e| tauri::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    println!("java_process_info: {}", java_process_info);
    let java_process_json_str = parse_jps_str(&java_process_info);
    Ok(java_process_json_str.to_string())
}


// 匹配class文件
#[tauri::command]
fn match_class_handle(javaFilePath: &str) -> Result<String, tauri::Error> {
    let config_content = get_config_file_content();
    let config: Config = serde_json::from_str(&config_content)
        .map_err(|e| tauri::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    println!("读取到配置：{}", config_content.as_str());

    let target_dir = confit_utils::get_executable_directory().join("data");
    let target_dir_str = target_dir.to_str().expect("路径不是有效的 UTF-8 字符串");


    match extract_class_files_from_directory(&config.project_path, javaFilePath, target_dir_str) {
        Ok(results) => {
            // 输出日志
            for class_info in &results {
                println!("Parent JAR File Name: {}", class_info.parent_jar_file_name);
                println!("JAR File Name: {}", class_info.jar_file_name);
                println!("Class File Name: {}", class_info.class_file_name);
                println!("Class File Path: {}", class_info.class_file_path);
                println!("-------------------------------------------");

            }

            // 将结果转换为 JSON 字符串
            serde_json::to_string(&results)
                .map_err(|e| tauri::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
        }
        Err(e) => {
            Err(tauri::Error::Io(e))
        }
    }
}

// 读取配置文件并返回
#[tauri::command]
fn read_config_event() -> Result<String, tauri::Error>{
    let config_content = get_config_file_content();
    // let config: Config = serde_json::from_str(&config_content)?;
    Ok(config_content)
}

// 开始热更新
#[tauri::command]
fn start_hot_update_handle(className: &str, javaProcessPid: &str) -> Result<String, tauri::Error> {
    let config_content = get_config_file_content();
    let config: Config = serde_json::from_str(&config_content).map_err(|e| tauri::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    // 获取执行路径
    let localClassPath: String = confit_utils::get_class_full_path(className);
    println!("className:{}, localClassPath:{}", className, localClassPath);

    let ssh = SshUtils::new(&config.server_ip, &config.server_username, &config.server_password)
        .map_err(|e| tauri::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    let remove_path = format!("/root/poodle/{}", className);

    ssh.upload_file(&localClassPath, &remove_path)
    .map_err(|e| tauri::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    // 执行热更新
    let hot_update_command = format!("echo -e 'redefine {}\nstop' | java -jar /root/arthas/arthas-boot.jar {}",remove_path, javaProcessPid);
    let output = ssh.exec_command(&hot_update_command)
        .map_err(|e| tauri::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;



    println!("Command output: {}", output);
    Ok(output)
}




fn main() {
    let ctx = tauri::generate_context!();
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![edit_save_handle, match_class_handle, read_config_event, start_hot_update_handle, get_java_process_list_handle])
        // .menu(
        //     tauri::Menu::os_default("Tauri Vue Template").add_submenu(Submenu::new(
        //         "Help",
        //         Menu::with_items([CustomMenuItem::new(
        //             "Online Documentation",
        //             "Online Documentation",
        //         )
        //         .into()]),
        //     )),
        // )
        // .on_menu_event(|event| {
        //     let event_name = event.menu_item_id();
        //     match event_name {
        //         "Online Documentation" => {
        //             let url = "https://github.com/Uninen/tauri-vue-template".to_string();
        //             shell::open(&event.window().shell_scope(), url, None).unwrap();
        //         }
        //         _ => {}
        //     }
        // })
        .setup(|_app| {
            #[cfg(debug_assertions)]
            {
                let main_window = _app.get_webview_window("main").unwrap();
                // let main_window = _app
                main_window.open_devtools();
            }
            Ok(())
        })
        .run(ctx)
        .expect("error while running tauri application");

        // tauri_app_lib::run()
}
