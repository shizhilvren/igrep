use crate::index::IndexBuilder;
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

    pub fn build(self) -> IndexBuilder {
        IndexBuilder { path: self.path }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let cfg = Config::new("path/to/config".to_string()).build();
        assert_eq!(cfg.path, "path/to/config");
    }
}
