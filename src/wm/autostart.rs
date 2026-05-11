use std::process::Command;

pub fn run_autostart(commands: &[String]) {
    let display = std::env::var("DISPLAY").ok();
    let xauth = std::env::var("XAUTHORITY").ok();

    for cmd in commands {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let Some((&bin, args)) = parts.split_first() else {
            continue;
        };

        let mut child = Command::new(bin);
        child.args(args);

        if let Some(display) = &display {
            child.env("DISPLAY", display);
        }

        if let Some(xauth) = &xauth {
            child.env("XAUTHORITY", xauth);
        }

        match child.spawn() {
            Ok(_) => println!("autostart: {cmd}"),
            Err(e) => eprintln!("autostart failed [{cmd}]: {e}"),
        }
    }
}
