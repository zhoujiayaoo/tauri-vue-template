use ssh2::Session;
use std::net::TcpStream;
use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};

pub struct SshUtils {
    session: Session,
}

impl SshUtils {
    // 建立与服务器的连接
    pub fn new(hostname: &str, username: &str, password: &str) -> Result<Self, ssh2::Error> {
        println!("尝试连接到 SSH 服务器: {}", hostname);
        let tcp = TcpStream::connect(hostname)
            .map_err(|e| {
                println!("连接失败: {}", e);
                ssh2::Error::from_errno(ssh2::ErrorCode::Session(e.raw_os_error().unwrap_or(-1)))
            })?;

        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;

        println!("正在验证用户名和密码...");
        session.userauth_password(username, password)?;

        if session.authenticated() {
            println!("认证成功.");
            Ok(SshUtils { session })
        } else {
            println!("认证失败.");
            Err(ssh2::Error::from_errno(ssh2::ErrorCode::Session(-1)))
        }
    }



    pub fn upload_file(&self, local_path: &str, remote_path: &str) -> Result<(), ssh2::Error> {
        println!("开始上传文件: {} 到 {}", local_path, remote_path);
    
        // 尝试打开本地文件
        let mut local_file = File::open(local_path).map_err(|e| {
            println!("打开本地文件失败: {}", e);
            ssh2::Error::from_errno(ssh2::ErrorCode::Session(e.raw_os_error().unwrap_or(-1)))
        })?;
    
        // 获取文件大小
        // let file_size = local_file.metadata()?.len();

        let file_size = match local_file.metadata() {
            Ok(metadata) => metadata.len(),
            Err(e) => {
                println!("获取文件元数据失败: {}", e);
                return Err(ssh2::Error::from_errno(ssh2::ErrorCode::Session(e.raw_os_error().unwrap_or(-1))));
            }
        };

        println!("本地文件大小: {} 字节", file_size);
    
        // 创建远程文件
        let mut remote_file = self.session.scp_send(Path::new(remote_path), 0o644, file_size, None)?;
    
        // 分块读取并写入文件
        let mut buffer = [0; 1024 * 32]; // 32KB 的缓冲区
        let mut total_written = 0;
        loop {
            let bytes_read = local_file.read(&mut buffer).map_err(|e| {
                println!("读取本地文件失败: {}", e);
                ssh2::Error::from_errno(ssh2::ErrorCode::Session(e.raw_os_error().unwrap_or(-1)))
            })?;
    
            if bytes_read == 0 {
                break;
            }
    
            remote_file.write_all(&buffer[..bytes_read]).map_err(|e| {
                println!("写入远程文件失败: {}", e);
                ssh2::Error::from_errno(ssh2::ErrorCode::Session(e.raw_os_error().unwrap_or(-1)))
            })?;
    
            total_written += bytes_read as u64;
            println!("已上传: {} / {} 字节", total_written, file_size);
        }
    
        if total_written == file_size {
            println!("文件上传成功");
        } else {
            println!("文件上传未完成，上传字节: {} / {}", total_written, file_size);
        }
    
        Ok(())
    }


    
    // 远程执行 SSH 命令
    pub fn exec_command(&self, command: &str) -> Result<String, ssh2::Error> {
        println!("执行远程命令: {}", command);
        let mut channel = self.session.channel_session()?;
        channel.exec(command)?;
        let mut s = String::new();
        channel.read_to_string(&mut s)
            .map_err(|e| {
                println!("读取命令输出失败: {}", e);
                ssh2::Error::from_errno(ssh2::ErrorCode::Session(e.raw_os_error().unwrap_or(-1)))
            })?;
        channel.wait_close()?;
        Ok(s)
    }
}
