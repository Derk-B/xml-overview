pub struct Config {
    pub minimize: bool,
}

impl Config {
    pub fn new(minimize: bool) -> Config {
        Config { minimize: minimize }
    }
}
