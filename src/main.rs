use std::{
    env::Args,
    error::Error,
    io::{stdout, Write},
    process::{exit, Command},
    sync::{Arc, Condvar, Mutex},
    time::Duration,
};

use crossterm::{
    cursor,
    style::Stylize,
    terminal::{self, disable_raw_mode, enable_raw_mode},
    ExecutableCommand, QueueableCommand,
};

mod settings;

use settings::{Settings, SettingsBuilder};

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
        let op_lines = count_lines(&output.stdout)?;
        let err_newlines = output.stderr.iter().filter(|i| **i == newline_code).count();
        stdout
            .queue(terminal::Clear(terminal::ClearType::FromCursorDown))
            .unwrap();
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
        let sum: u16 = (op_lines + err_newlines as i16) as u16;
        stdout.queue(cursor::MoveUp(sum)).unwrap();
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
        let rest = args.collect::<Vec<_>>().join(" ");
        (
            SettingsBuilder::default().build(),
            format!("{first} {rest}"),
        )
    };
    (settings, command)
}

fn count_lines(buf: &[u8]) -> Result<i16, Box<dyn Error>> {
    let terminal_size = terminal::size()?;
    let terminal_width = terminal_size.0;
    let terminal_height = terminal_size.1;
    let newline_code = 10;
    let mut count = 0;
    let mut line_len = 0;
    for ch in buf {
        if *ch == newline_code {
            // so till now is one line
            count += 1;
            count += line_len / terminal_width;
            line_len = 0;
        } else {
            line_len += 1;
        }
    }

    let cursor_position = cursor::position()?;
    let cursor_y = cursor_position.1;
    if cursor_y + count > terminal_height {
        // scroll
        let offset = cursor_y + count - terminal_height;
        Ok((count + offset) as i16)
    } else {
        Ok(count as i16)
    }
}
