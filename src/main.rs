mod config;
mod wm;
mod x11;

fn main() {
    let config = config::Config::load();
    println!(
        "Config loaded: gap={}, terminal={}",
        config.gap, config.terminal
    );
    x11::event::start_x11_wm(config);
}
