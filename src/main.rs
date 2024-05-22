use std::{
    collections::HashSet,
    process::{Child, Command},
};

use anyhow::{Context, Ok};

struct Application;

impl Application {
    /// Get all the programs in the system (and leanking the strings... oops)
    fn get_programs() -> anyhow::Result<Vec<&'static str>> {
        let output = Command::new("bash")
            .args(["-c", "compgen -c"])
            .output()
            .context("running compgen")?
            .stdout;

        let bindings =
            String::from_utf8(output).context("parsing stdout buffer to UTF-8 String")?;

        // Split the programs by newline and remove duplicates
        let programs: HashSet<String> = bindings
            .lines()
            .map(|program| program.to_string())
            .collect();

        Ok(programs
            .into_iter()
            .map(|program| program.leak() as &'static str)
            .collect())
    }

    /// Run a command for the launcher_program
    fn run_bash_command(launcher_program: &str, command: &str) -> anyhow::Result<Child> {
        Command::new(launcher_program)
            .args(["-c", command])
            .spawn()
            .with_context(|| format!("spawn {} program", launcher_program))
    }
}

// pub fn main() -> anyhow::Result<()> {
//     const FONT_SIZE: i32 = 32;
//     const WINDOW_WIDTH: i32 = 500;
//     const WINDOW_HEIGHT: i32 = 800;
//
//     Ok(())
// }

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    ui.on_request_increase_value({
        let ui_handle = ui.as_weak();
        move || {
            let ui = ui_handle.unwrap();
            ui.set_counter(ui.get_counter() + 1);
        }
    });

    ui.run()
}
