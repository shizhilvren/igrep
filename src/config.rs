use std::io;

use crate::index::{self, IndexBuilder};
pub struct Config {
    path: String,
}

impl Config {
    pub fn new(path: String) -> Self {
        Config { path }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn build(self) -> Result<IndexBuilder,io::Error> {
        IndexBuilder::new(self.path.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let cfg = Config::new("/usr".to_string()).build().unwrap();
        assert_eq!(cfg.path.path(), "/usr");
    }
}
