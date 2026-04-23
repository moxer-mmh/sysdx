mod app;
mod config;
mod filter;
mod input;
mod systemd;
mod theme;
mod ui;
mod units;

use anyhow::Result;
use app::App;
use config::Config;
use systemd::Scope;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("sysdx {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return Ok(());
    }

    let scope = if args.iter().any(|a| a == "--system") {
        Scope::System
    } else {
        Scope::User
    };

    let config = Config::load()?;
    let app = App::new(config, scope);

    let mut terminal = ratatui::init();
    let result = app.run(&mut terminal);
    ratatui::restore();

    result
}

fn print_help() {
    println!(
        "sysdx {} — systemd unit manager TUI

USAGE:
    sysdx [OPTIONS]

OPTIONS:
    --user      Show user units (default)
    --system    Show system units
    --version   Print version
    --help      Print this help

KEYBINDS (defaults):
    j/k         Navigate down/up
    g/G         Go to top/bottom
    ctrl-d/u    Half-page down/up
    /           Fuzzy filter
    enter       Open action menu
    tab         Toggle user/system scope
    l           View journal logs
    q           Quit

CONFIG:
    ~/.config/sysdx/config.toml
",
        env!("CARGO_PKG_VERSION")
    );
}
