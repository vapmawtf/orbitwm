use std::process::Command;

pub fn run_autostart(commands: &[String]) {
    for cmd in commands {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        let Some((&bin, args)) = parts.split_first() else {
            continue;
        };

        match Command::new(bin).args(args).spawn() {
            Ok(_) => println!("autostart: {cmd}"),
            Err(e) => eprintln!("autostart failed [{cmd}]: {e}"),
        }
    }
}
