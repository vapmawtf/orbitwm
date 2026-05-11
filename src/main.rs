mod config;
mod wm;
mod x11;

fn main() {
    if std::env::args()
        .skip(1)
        .any(|arg| arg == "-v" || arg == "--version")
    {
        println!("orbitwm {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let config = config::Config::load();

    // autostart
    wm::autostart::run_autostart(&config.autostart);

    // state init
    let mut state = wm::state::WMState::new();

    // X11 init + event loop
    x11::event::start_x11_wm(&config, &mut state);
}
