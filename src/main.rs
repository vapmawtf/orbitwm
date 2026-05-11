mod config;
mod wm;
mod x11;

fn main() {
    let config = config::Config::load();

    // autostart
    wm::autostart::run_autostart(&config.autostart);

    // state init
    let mut state = wm::state::WMState::new();

    // X11 init + event loop
    x11::event::start_x11_wm(&config, &mut state);
}
