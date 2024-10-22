use std::{
    env::Args,
    error::Error,
    io::{stdout, Write},
    process::{exit, Command},
    sync::{Arc, Condvar, Mutex},
    time::Duration,
};

use crossterm::{cursor, style::Stylize, terminal, ExecutableCommand, QueueableCommand};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();
    let program_name = args.next().unwrap();
    let (settings, command) = parse_flags(args);
    if command.len() == 0 {
        println!("{}\n", "simple-watch".bold());
        println!("Usage: {program_name} [-i <I>|--interavl=<I>] <command>");
        exit(2);
    }
    let pair = Arc::new((Mutex::new(true), Condvar::new()));
    let pair2 = Arc::clone(&pair);
    ctrlc::set_handler(move || {
        let (lock, cvar) = &*pair2;
        let mut running = lock.lock().unwrap();
        *running = false;
        cvar.notify_one();
    })
    .expect("Error setting Ctrl+C handler");

    let mut stdout = stdout();

    stdout.execute(cursor::Hide).unwrap();
    loop {
        let output = Command::new("sh").arg("-c").arg(&command).output().unwrap();
        let newline_code = 10;
        let op_newlines = output.stdout.iter().filter(|i| **i == newline_code).count();
        let err_newlines = output.stderr.iter().filter(|i| **i == newline_code).count();
        // todo: maybe remove last newline character?
        stdout.write_all(&output.stdout).unwrap();
        stdout.write_all(&output.stderr).unwrap();
        stdout.flush().unwrap();
        let (lock, cvar) = &*pair;
        let timeout_result = cvar
            .wait_timeout_while(lock.lock().unwrap(), settings.interval, |&mut running| {
                running
            })
            .unwrap();
        if !timeout_result.1.timed_out() {
            // it didn't time out - i.e. condition changed so we break
            break;
        }
        let sum: u16 = (op_newlines + err_newlines) as u16;
        stdout.queue(cursor::MoveUp(sum)).unwrap();
        stdout
            .queue(terminal::Clear(terminal::ClearType::FromCursorDown))
            .unwrap();
    }
    stdout.execute(cursor::Show).unwrap();
    Ok(())
}

fn parse_flags(mut args: Args) -> (Settings, String) {
    let first = args.next().unwrap_or("".to_string());
    let (settings, command) = if first.starts_with("-") {
        // flag
        let settings = if first.starts_with("--") {
            // long
            let flag = first.chars().skip(2).collect::<String>();
            let (flag, value) = if let Some((before_equals, value)) = flag.split_once("=") {
                (before_equals.to_string(), value.to_string())
            } else {
                (flag, args.next().unwrap())
            };
            SettingsBuilder::default()
                .add_long_arg(&flag, &value)
                .build()
        } else {
            let flag = first.chars().skip(1).collect::<String>();
            let value = args.next().unwrap();
            SettingsBuilder::default()
                .add_short_arg(&flag, &value)
                .build()
        };
        (settings, args.collect::<Vec<_>>().join(" "))
    } else {
        (SettingsBuilder::default().build(), first)
    };
    (settings, command)
}

#[derive(Debug)]
struct Settings {
    interval: Duration,
}

#[derive(Default)]
struct SettingsBuilder {
    interval: Option<Duration>,
}

impl SettingsBuilder {
    fn add_short_arg(mut self, arg: &str, value: &str) -> Self {
        match arg {
            "i" => {
                let m: u64 = value.parse().unwrap();
                self.interval = Some(Duration::from_millis(m));
            }
            _ => panic!("unknown arg"),
        }
        self
    }
    fn add_long_arg(mut self, arg: &str, value: &str) -> Self {
        match arg {
            "interval" => {
                let m: u64 = value.parse().unwrap();
                self.interval = Some(Duration::from_millis(m));
            }
            _ => panic!("unknown arg"),
        }
        self
    }

    fn build(self) -> Settings {
        Settings {
            interval: self.interval.unwrap_or(Duration::from_millis(200)),
        }
    }
}
