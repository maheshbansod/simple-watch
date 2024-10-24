use std::time::Duration;

#[derive(Debug)]
pub struct Settings {
    pub interval: Duration,
}

#[derive(Default)]
pub struct SettingsBuilder {
    interval: Option<Duration>,
}

impl SettingsBuilder {
    pub fn add_short_arg(mut self, arg: &str, value: &str) -> Self {
        match arg {
            "i" => {
                let m: u64 = value.parse().unwrap();
                self.interval = Some(Duration::from_millis(m));
            }
            _ => panic!("unknown arg"),
        }
        self
    }
    pub fn add_long_arg(mut self, arg: &str, value: &str) -> Self {
        match arg {
            "interval" => {
                let m: u64 = value.parse().unwrap();
                self.interval = Some(Duration::from_millis(m));
            }
            _ => panic!("unknown arg"),
        }
        self
    }

    pub fn build(self) -> Settings {
        Settings {
            interval: self.interval.unwrap_or(Duration::from_millis(200)),
        }
    }
}
