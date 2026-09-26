use std::process::ExitCode;

use knave_renderer::WgpuRenderer;
use knave_ui::UiScene;
use knave_wayland::ShellRole;

fn usage() -> &'static str {
    "usage: knave-shell [--check] [bar|overview]"
}

fn check(role: ShellRole) {
    let scene = match role {
        ShellRole::Bar => UiScene::bar(1, 1920.0, 36.0),
        ShellRole::Overview => UiScene::overview(1, 1920.0, 1080.0),
    };
    let renderer = WgpuRenderer::new();
    let render_list = renderer.prepare(&scene);
    println!(
        "knave-shell role={} revision={} commands={}",
        match role {
            ShellRole::Bar => "bar",
            ShellRole::Overview => "overview",
        },
        render_list.revision,
        render_list.commands.len()
    );
}

fn run() -> Result<(), String> {
    let mut check_only = false;
    let mut role = ShellRole::Bar;

    for argument in std::env::args().skip(1) {
        match argument.as_str() {
            "--check" => check_only = true,
            "--help" | "-h" => {
                println!("{}", usage());
                return Ok(());
            }
            value => role = ShellRole::parse(value).ok_or_else(|| usage().to_string())?,
        }
    }

    if check_only {
        check(role);
        return Ok(());
    }

    knave_wayland::run(role).map_err(|error| error.to_string())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("knave-shell: {error}\n\n{}", usage());
            ExitCode::FAILURE
        }
    }
}
