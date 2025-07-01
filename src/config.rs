
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

}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     #[test]
//     fn it_works() {
//         let cfg = Config::new("/usr".to_string()).build().unwrap();
//         assert_eq!(cfg.path.path(), "/usr");
//     }
// }
