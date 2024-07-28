use std::path::PathBuf;

use clap::{command, Parser, ValueEnum};

mod compress;
use compress::Zipper;


#[derive(Parser)]
#[command(about = "进行文件压缩")]
#[command(long_about = "对文件或文件夹进行单次压缩")]
pub struct ZipCli {
    /// 压缩文件或目录
    input: PathBuf,

    /// 输出文件夹
    #[arg(default_value = "./")]
    output: PathBuf,

    /// 压缩密码
    #[arg(short, long)]
    password: Option<String>,

    ///压缩模式
    #[arg(short, long,default_value="deflated")]
    method: Method,

    /// 压缩文件扩展名
    #[arg(short, long)]
    extend:Option<String>,
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

    fn get_output(&self) -> PathBuf {
        let extend = if let Some(ext) = self.extend.clone() {
            ext
        }else{
            match self.method {
                Method::Bzip2 => String::from("bz2"),
                _ => String::from("zip"),
            }
        };

        let mut output = self.output.clone();
        if let None = output.extension() {
            let name = self.input.file_stem().unwrap();
            output.push(name);
        }
            
        output.set_extension(extend);
        
        output
    }
    
    pub fn run(&self) {
        let method = self.get_method();
        let output = self.get_output();

        let mut zipper = Zipper::new(self.input.clone(),output.clone(), self.password.clone(), method);
        zipper.compress(self.input.clone());
        zipper.finish();

        println!("压缩完成: {}",output.display());
    }
}



