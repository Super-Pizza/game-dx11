#[macro_use]
mod utils;
mod app;
use clap::clap_app;
use std::process::exit;
fn main() {
    let result = {
        let clap_app = clap_app! {App =>
            (version: "0.1.0")
            (author: "Fabien Cournoyer <fabien@cournoyer.club")
            (about: "A Minecraft-like game, but MUCH faster!")
            (@arg VSYNC: -v --vsync {is_bool} "Enable VSync(Vertical Synchronization)")
            (@arg WIDTH:
                -w
                --width
                {is_num}
                requires[HEIGHT]
                conflicts_with[MAXMIZED FULLSCREEN]
                "Set window width")
            (@arg HEIGHT:
                -h
                --height
                {is_num}
                requires[WIDTH]
                conflicts_with[MAXMIZED FULLSCREEN]
                "Set window height")
            (@arg FULLSCREEN: -f --fullscreen conflicts_with[MAXMIZED] "Start in fullscreen")
            (@arg MAXIMIZED: -m --maximized "Start maximized")
        }
        .get_matches();
        let state = if clap_app.is_present("FULLSCREEN") {
            app::WindowState::Fullscreen
        } else if clap_app.is_present("MAXIMIZED") {
            app::WindowState::Maximized
        } else if let (Some(w), Some(h)) = (clap_app.value_of("WIDTH"), clap_app.value_of("HEIGHT"))
        {
            app::WindowState::Windowed(w.parse().unwrap(), h.parse().unwrap())
        } else {
            app::WindowState::Windowed(800, 600)
        };
        let vsync = clap_app
            .value_of("VSYNC")
            .unwrap_or("true")
            .parse::<bool>()
            .unwrap();
        let flags = app::Flags { state, vsync };
        match app::App::init(flags, String::from("my_game")) {
            Ok(mut a) => a.run(),
            Err(e) => e,
        }
    };
    exit(result);
}
fn is_bool(v: String) -> Result<(), String> {
    if v == "true" || v == "false" {
        Ok(())
    } else {
        Err(String::from("Must be boolean"))
    }
}
fn is_num(v: String) -> Result<(), String> {
    match v.parse::<u16>() {
        Ok(_) => Ok(()),
        Err(_) => Err(String::from("Must be integer")),
    }
}
