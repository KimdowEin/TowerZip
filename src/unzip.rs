use std::{collections::VecDeque, fmt::Display, fs, io::{self, Read}, path::PathBuf};

use clap::{command, Parser};

mod decompress;

#[derive(Parser)]
#[command(about="进行文件解压",)]
#[command(long_about = "对文件进行多次解压直到不包含压缩包(回避jar,apk等特殊文件格式)")]

pub struct UnZipCli {
    ///要解压的文件的路径
    input: PathBuf,

    ///(可选)输出位置
    output: Option<PathBuf>,

    ///(可选)压缩密码
    #[arg(long, short)]
    password: Option<String>,

    ///解压成功是否删除源文件,默认关闭
    #[arg(short,long,action = clap::ArgAction::SetTrue)]
    mode: bool,
}
impl UnZipCli {
    pub fn get_mode(&self) -> bool {
        self.mode
    }
    pub fn run(self){
        let mode = self.get_mode().clone();

        let mut task_list = task_list_init(self);

        let mut success = 0;
        let mut failed = 0;

        while !task_list.is_empty() {
            let result = task_list.pop_front().unwrap().run();
            for task in result {
                match task.state() {
                    CompleteState::Success => {
                        success += 1;
                        println!("Success:{}", task);
                        if mode {
                            let _ = task.dele();
                        }
                    }
                    CompleteState::Retry => task_list.push_back(task),
                    CompleteState::Failed => {
                        failed += 1;
                        println!("Failed:{}", task);
                    }
                }
            }
        }
        
        println!("Success: {} Failed: {}", success, failed);
    }
}

#[derive(Clone, Copy)]
enum CompleteState {
    Success,
    Retry,
    Failed,
}

enum TaskType {
    Zip,
    BZip,
    Rar,
    File,
}

struct Task {
    input: PathBuf,
    output: PathBuf,
    password: Option<String>,
    task_type: TaskType,
    task_state: CompleteState,
}
impl Task {
    fn scan(input: PathBuf) -> TaskType {
        //扫描文件类型
        let mut file = fs::File::open(&input).unwrap();
        let mut magic_num = [0u8; 8];
        let _ = file.read(&mut magic_num);

        match magic_num {
            [0x50, 0x4B, 0x03, 0x04, _, _, _, _] => {
                match input.extension() {
                    Some(ext)=>{
                        if ext == "apk" || ext == "jar" {
                            TaskType::File
                        }else{
                            TaskType::Zip
                        }
                    }
                    None=>TaskType::Zip
                }
            },
            [0x42, 0x5A, 0x68, _, _, _, _, _] => TaskType::BZip,
            [0x52, 0x61, 0x72, 0x21, _, _, _, _] => TaskType::Rar,
            _ => TaskType::File,
        }
    }
    fn new(input: PathBuf, output: PathBuf, password: Option<String>) -> Self {
        Self {
            input: input.clone(),
            output,
            password,
            task_type: Task::scan(input),
            task_state: CompleteState::Retry,
        }
    }
    pub fn run(self) -> VecDeque<Task> {
        println!("Begin:{}", self);
        // 执行解压操作
        match self.task_type {
            TaskType::Zip | TaskType::BZip => decompress::zip::zip_decompress(self),
            TaskType::Rar | TaskType::File => decompress::file::copy(self),
        }
    }

    pub fn dele(self) -> Result<(), std::io::Error> {
        fs::remove_file(self.input)
    }

    fn success(&mut self) {
        self.task_state = CompleteState::Success
    }

    fn _retry(&mut self) {
        self.task_state = CompleteState::Retry
    }

    fn failed(&mut self) {
        self.task_state = CompleteState::Failed
    }
    pub fn state(&self) -> CompleteState {
        self.task_state
    }
}
impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " {} -> {}", self.input.display(), self.output.display())
    }
}

fn task_list_init(cli: UnZipCli) -> VecDeque<Task> {
    let output_root = if let Some(output) = cli.output {
        if !output.exists(){
            println!("output path not exist, create it or not [y/n]");
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            if input.trim().starts_with('y'){
                fs::create_dir_all(&output).unwrap();
            }else {
                panic!("output path not exist");
            }
        }
        output
    } else {
        PathBuf::from("./")
    };
    let input_root = cli.input;

    //初始化任务列表
    let mut task_list = VecDeque::new();
    let walk = walkdir::WalkDir::new(&input_root);

    for entry in walk {
        let entry = entry.unwrap();
        let path = entry.path();

        let mut output = output_root.clone();
        if path.is_file() {
            output.push(path.strip_prefix(input_root.clone()).unwrap().to_path_buf()); //路径拼接到输出目录下
            if input_root.is_dir() {
                output.pop();
            } //去除文件名
            task_list.push_back(Task::new(
                path.to_path_buf(),
                output.clone(),
                cli.password.clone(),
            ));
        }
    }
    task_list
}
