use super::{Task, TaskType};
use std::{
    collections::VecDeque,
    fs,
    io::{Read, Write},
};
pub mod zip {
    use super::*;
    use ::zip::ZipArchive;
    use indicatif::ProgressBar;
    pub fn zip_decompress(mut task: Task) -> VecDeque<Task> {
        let mut zip = ZipArchive::new(fs::File::open(&task.input).unwrap()).unwrap();

        let mut report = VecDeque::new();

        let bar = ProgressBar::new(zip.len() as u64);

        for index in 0..zip.len() {
            //解压
            let zipfile = match task.password.clone() {
                None => zip.by_index(index),
                Some(password) => zip.by_index_decrypt(index, password.as_bytes()),
            };

            //是否成功
            let mut zipfile = match zipfile {
                Ok(zipfile) => zipfile,
                Err(_) => {
                    task.failed();
                    report.push_back(task);
                    return report;
                }
            };

            //写入文件
            let outpath = task.output.join(zipfile.enclosed_name().unwrap());

            if zipfile.is_dir() {
                let _ = fs::create_dir_all(&outpath);
                continue;
            }

            fs::create_dir_all(&outpath.parent().unwrap()).unwrap();
            let mut file = fs::File::create(&outpath).unwrap();
            let mut buffer = Vec::new();
            zipfile.read_to_end(&mut buffer).unwrap();

            let _ = file.write_all(&buffer);

            //检查输出文件
            match Task::scan(outpath.clone()) {
                TaskType::File => {}
                _ => report.push_back(Task::new(
                    outpath.clone(),
                    outpath.parent().unwrap().to_path_buf(),
                    task.password.clone(),
                )),
            }

            //进度条
            bar.inc(1);
        }

        task.success();
        report.push_front(task);
        report
    }
}

pub mod file {
    use super::*;

    pub fn copy(mut task: Task) -> VecDeque<Task> {
        let output = task.output.join(task.input.file_name().unwrap());

        fs::copy(&task.input, output).unwrap();

        task.success();

        let mut report = VecDeque::new();
        report.push_back(task);
        report
    }
}
