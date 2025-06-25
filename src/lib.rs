pub mod config;
pub mod index;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn it_works_with_config() {
        let cfg = config::Config::new(
            "path/to/config".to_string(),
        );
        assert_eq!(cfg.path(), "path/to/config");
    }
}
