use std::process::Command;

pub fn run_autostart(commands: &[String]) {
    let display = std::env::var("DISPLAY").ok();
    let xauth = std::env::var("XAUTHORITY").ok();
    // Under nested X servers like Xephyr, starting a compositor (e.g. picom)
    // often fails or can break rendering/redirect behavior.
    let is_nested_x = display
        .as_deref()
        .and_then(|d| d.strip_prefix(':'))
        .and_then(|d| d.split('.').next())
        .and_then(|n| n.parse::<u32>().ok())
        .is_some_and(|n| n > 0);

    for cmd in commands {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let Some((&bin, args)) = parts.split_first() else {
            continue;
        };

        if is_nested_x && bin == "picom" {
            eprintln!("autostart: skipping '{cmd}' on nested DISPLAY");
            continue;
        }

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
