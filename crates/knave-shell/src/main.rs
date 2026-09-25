use std::process::ExitCode;

use knave_renderer::WgpuRenderer;
use knave_ui::UiScene;

fn usage() -> &'static str {
    "usage: knave-shell [bar|overview]"
}

fn run() -> Result<(), String> {
    let role = std::env::args().nth(1).unwrap_or_else(|| "bar".to_string());
    let scene = match role.as_str() {
        "bar" => UiScene::bar(1, 1920.0, 36.0),
        "overview" => UiScene::overview(1, 1920.0, 1080.0),
        _ => return Err(usage().into()),
    };
    let renderer = WgpuRenderer::new();
    let render_list = renderer.prepare(&scene);
    println!(
        "knave-shell role={role} revision={} commands={}",
        render_list.revision,
        render_list.commands.len()
    );
    Ok(())
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
