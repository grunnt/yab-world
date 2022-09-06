use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Assets {
    assets_path: PathBuf,
}

impl Assets {
    pub fn default() -> Assets {
        Assets::new("assets")
    }

    pub fn new(path: &str) -> Assets {
        let exe_file_name = ::std::env::current_exe().unwrap();
        let exe_path = exe_file_name.parent().unwrap().to_path_buf();
        Assets {
            assets_path: exe_path.join(path),
        }
    }

    pub fn exists(&self) -> bool {
        self.assets_path.exists() && self.assets_path.is_dir()
    }

    pub fn path(&self, file: &str) -> PathBuf {
        self.assets_path.join(file)
    }
}
