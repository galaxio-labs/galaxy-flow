use std::{
    fs::{File, create_dir_all},
    path::Path,
};

use anyhow::Context;
use orion_infra::path::ensure_path;
//use crate::friconf::ensure_directory_exists;
use time::{OffsetDateTime, format_description};

use crate::ability::prelude::ExecOut;

// 任务直接结果本地化
pub fn task_local_report(out: ExecOut) {
    let datetime = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    let format =
        format_description::parse_borrowed::<3>("[year]-[month]-[day] [hour]:[minute]:[second]");
    let now = datetime.format(&format.unwrap()).unwrap();
    let dir_path = "_gal/.report";
    let path = Path::new(dir_path);

    // report目录不存在则创建
    if let Err(e) = create_dir_all(path) {
        println!("Failed to create directory '{dir_path}': {e}",);
        return;
    }

    let file_name = format!("{dir_path}/task_{now}.yaml",);

    // 创建task结果存储文件
    match File::create(&file_name) {
        Ok(_) => {
            // 将配置数据序列化为 yaml 字符串
            let toml = serde_yaml::to_string(&out).unwrap();
            let path = Path::new(&file_name);

            // 确保目录存在
            if let Err(e) = ensure_path(path) {
                println!("Failed to ensure directory exists: {e}",);
            };

            // 将 yaml 字符串写入文件
            if let Err(e) = std::fs::write(path, toml)
                .with_context(|| format!("write toml file : {file_name}",))
            {
                println!("Failed to write toml file: {e}",);
            }
        }
        Err(_) => {
            println!("Failed to create file: {file_name}",);
        }
    }
}
