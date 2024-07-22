use std::{fs, io::{self, Write}, path::PathBuf};

use zip::{unstable::write::FileOptionsExt, write::SimpleFileOptions, ZipWriter};

pub struct Zipper {
    zip:ZipWriter<fs::File>,
    ///压缩方式
    options: SimpleFileOptions,
    ///目标所在根目录
    root: PathBuf,
}
impl Zipper { 
    pub fn new(input:PathBuf,output: PathBuf, password: Option<String>,method:zip::CompressionMethod) -> Self {
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

    pub fn compress(&mut self,input:PathBuf){
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

    pub fn finish(self){
        self.zip.finish().unwrap();
    }
}
