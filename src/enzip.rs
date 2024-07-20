use std::{fs, io::{self, Write}, path::PathBuf};

use clap::{command, Parser, ValueEnum};
use zip::{unstable::write::FileOptionsExt, write::SimpleFileOptions, ZipWriter};

mod compress;

#[derive(Parser)]
#[command(about = "压缩", long_about = "压缩")]
pub struct ZipCli {
    /// 压缩文件或目录
    input: PathBuf,

    /// 输出位置,带文件名的路径
    #[arg(default_value = "./out.zip")]
    output: PathBuf,

    /// 压缩密码
    #[arg(short, long)]
    password: Option<String>,

    ///压缩模式
    #[arg(short, long,default_value="deflated")]
    method: Method,

}
#[derive(ValueEnum,Clone, Copy)]
enum Method {
    Stored,
    Deflated,
    Deflate64,
    Bzip2,
}

impl ZipCli {
    fn get_method(&self) -> zip::CompressionMethod {
        match self.method {
            Method::Stored => zip::CompressionMethod::Stored,
            Method::Deflated => zip::CompressionMethod::Deflated,
            Method::Deflate64 => zip::CompressionMethod::Deflate64,
            Method::Bzip2 => zip::CompressionMethod::Bzip2,
        }
    }
    
    pub fn run(&self) {
        let method = self.get_method();

        let mut zipper = Zipper::new(self.input.clone(),self.output.clone(), self.password.clone(), method);
        zipper.compress(self.input.clone());
        zipper.finish();

        println!("压缩完成: {}",self.output.display());
    }
}

struct Zipper {
    zip:ZipWriter<fs::File>,
    ///压缩方式
    options: SimpleFileOptions,
    ///目标所在根目录
    root: PathBuf,
}
impl Zipper { 
    fn new(input:PathBuf,output: PathBuf, password: Option<String>,method:zip::CompressionMethod) -> Self {
        let root = if let Some(parent) = input.parent() {
            parent.to_path_buf()
        }else{
            input
        };

        let inner = fs::File::create(output).unwrap();
        let zip: ZipWriter<fs::File> = ZipWriter::new(inner);

        let options = SimpleFileOptions::default().compression_method(method);
        let options = match password {
            Some(password) => {
                options.with_deprecated_encryption(password.as_bytes())
            }
            None => options
        };

        Self {
            zip,
            options,
            root,
        }
    }

    fn add_file(&mut self,filepath:PathBuf){
        let path = filepath.strip_prefix(&self.root).unwrap();
        let path = path.to_str().unwrap().replace('\\', "/");

        if filepath.is_dir() {
            self.zip.add_directory(path, self.options).unwrap();
        }else{
            let file = fs::read(filepath).unwrap();
            self.zip.start_file(path, self.options).unwrap();
            self.zip.write_all(&file).unwrap();
        }
    }

    fn compress(&mut self,input:PathBuf){
        let walk = walkdir::WalkDir::new(input);
        for entry in walk {
            let entry = entry.unwrap();
            let path = entry.path();

            print!("正在压缩: {}", path.display());
            io::stdout().flush().unwrap();
            self.add_file(path.to_path_buf());
            print!("\r\x1b[K");
            io::stdout().flush().unwrap();
        }
    }

    fn finish(self){
        self.zip.finish().unwrap();
    }
}
