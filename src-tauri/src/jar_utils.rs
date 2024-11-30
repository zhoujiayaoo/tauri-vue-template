use serde::{Deserialize, Serialize};
use std::fmt::format;
use std::fs;
use std::fs::File;
use std::io::{self, Cursor, Read};
use std::path::{Path, PathBuf};
use zip::read::ZipArchive;

// 定义一个结构体来存储 class 文件的信息
#[derive(Serialize, Deserialize)]
pub struct ClassInfoDTO {
    pub parent_jar_file_name: String, // 最外层的 JAR 文件名
    pub jar_file_name: String,        // 当前 JAR 文件名
    pub class_file_name: String,      // class 文件名
    pub class_file_path: String,      // class 文件的路径
    pub java_process_list_str:  String
}

// 递归处理 JAR 归档文件
pub fn process_jar_archive<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    class_file_fragment: &str,
    base_dir: &Path,            // 基础目录，用于计算相对路径
    current_path: &Path,        // 当前路径，相对于基础目录
    parent_jar_file_name: &str, // 最外层的 JAR 文件名
    output_dir: &str,
) -> io::Result<Vec<ClassInfoDTO>> {
    // println!("处理 JAR 归档: {:?}", current_path);
    let mut results = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_name = file.name().to_string();

        if file_name.ends_with(".jar") {
            // 处理嵌套的 JAR 文件
            let mut jar_contents = Vec::new();
            file.read_to_end(&mut jar_contents)?;
            let reader = Cursor::new(jar_contents);
            let mut nested_archive = ZipArchive::new(reader)?;

            let nested_path = current_path.join(&file_name);
            results.extend(process_jar_archive(
                &mut nested_archive,
                class_file_fragment,
                base_dir,
                &nested_path,
                parent_jar_file_name,
                output_dir
            )?);
        } else if file_name.contains(class_file_fragment) {
            // let output_path: PathBuf = base_dir.join(&file_name);
            // println!("输出路径为：{}", output_path.to_string_lossy());
            // let mut outfile = File::create(&output_path)?;
            // io::copy(&mut file, &mut outfile)?;

            // 获取文件的基本名称（不包含路径）
            let file_base_name = Path::new(&file_name)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();

            // 构造输出路径：基础目录 + 文件基本名称
            let output_path = Path::new(output_dir);
            let output_path_buf: PathBuf = output_path.join(&file_base_name);
            println!("输出路径为：{}", output_path_buf.to_string_lossy());
            let mut outfile = File::create(&output_path_buf)?;
            io::copy(&mut file, &mut outfile)?;

            // 构造 class 文件的相对路径
            let relative_path = current_path.join(&file_name);

            let class_info = ClassInfoDTO {
                parent_jar_file_name: parent_jar_file_name.to_string(),
                jar_file_name: current_path.to_string_lossy().into_owned(),
                class_file_name: file_base_name,
                class_file_path: relative_path.to_string_lossy().into_owned(),
                java_process_list_str: format!("")
            };

            results.push(class_info);
            // println!("找到匹配的 class 文件: {}", file_name);
        }
    }

    Ok(results)
}

// 递归处理目录和 JAR 文件
pub fn process_directory(
    path: &Path,
    class_file_fragment: &str,
    base_dir: &Path, // 基础目录，用于计算相对路径
    output_dir: &str
) -> io::Result<Vec<ClassInfoDTO>> {
    // println!("处理目录: {:?}", path);
    let mut results = Vec::new();

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // 递归处理子目录
                results.extend(process_directory(&path, class_file_fragment, base_dir, output_dir)?);
            } else if path.is_file()
                && path.extension().and_then(std::ffi::OsStr::to_str) == Some("jar")
            {
                // 处理 JAR 文件
                let jar_file = File::open(&path)?;
                let mut archive = ZipArchive::new(jar_file)?;

                let relative_path = path.strip_prefix(base_dir).unwrap_or(&path);
                let parent_jar_file_name = relative_path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();

                results.extend(process_jar_archive(
                    &mut archive,
                    class_file_fragment,
                    base_dir,
                    relative_path,
                    &parent_jar_file_name,
                    output_dir,
                )?);
            }
        }
    }

    Ok(results)
}

// 从指定目录的 JAR 文件中提取 class 文件
pub fn extract_class_files_from_directory(
    dir_path: &str,
    java_path: &str,
    target_dir: &str,
) -> io::Result<Vec<ClassInfoDTO>> {
    println!("提取目录中的 class 文件: {}", dir_path);
    let class_file_path_fragment = java_path
        .split(':')
        .next()
        .unwrap()
        .replace(".java", ".class");

    let base_dir = Path::new(dir_path);
    process_directory(base_dir, &class_file_path_fragment, base_dir, target_dir)
}


