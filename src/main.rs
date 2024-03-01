use raylib::{ffi::GetCharPressed, prelude::*};
use std::{
    collections::HashSet,
    io,
    process::{Child, Command},
};

struct Application;

impl Application {
    /// Get all the programs in the system
    fn get_programs() -> Vec<String> {
        let output = Command::new("bash")
            .args(["-c", "compgen -c"])
            .output()
            .unwrap()
            .stdout;

        let bindings = String::from_utf8(output).unwrap();

        // Split the programs by newline and remove duplicates
        let programs: HashSet<String> = bindings
            .lines()
            .map(|program| program.to_string())
            .collect();

        programs.into_iter().collect()
    }

    /// Run a command for the launcher_program
    fn run_bash_command(state: &AppConfig, command: &str) -> io::Result<Child> {
        Command::new(&state.launcher_program)
            .args(["-c", command])
            .spawn()
    }
}

#[derive(Default)]
struct AppConfig {
    title: String,
    width: i32,
    height: i32,
    font_size: i32,
    programs: Vec<String>,
    launcher_program: String,
}

struct App {
    config: AppConfig,
    selected_idx: usize,
    programs: Vec<String>,
    max: usize,
    search_bar: String,
    programs_count: usize,
}

impl App {
    const TEXT_PADDING: i32 = 10;

    fn new(config: AppConfig) -> Self {
        let programs = config.programs.clone();

        let search_bar = String::new();
        let selected_idx = 0;

        // The maximum number of programs showing
        let max = ((config.height / config.font_size) - 2) as usize;

        // The number of programs showing
        let programs_count = programs.len().min(max);

        Self {
            config,
            selected_idx,
            programs,
            programs_count,
            max,
            search_bar,
        }
    }

    /// Draw the application list
    fn draw_application_list(&mut self, drawer: &mut RaylibDrawHandle<'_>) {
        for (idx, program) in self.programs.iter().take(self.max).enumerate() {
            let y_padding = (idx as i32 * self.config.font_size) + self.config.font_size * 2;

            if idx == self.selected_idx {
                // Draw the ligtgray background for the selected program
                drawer.draw_rectangle(
                    0,
                    y_padding,
                    self.config.width,
                    self.config.font_size,
                    Color::LIGHTGRAY,
                );
                // Draw the selected program in black
                drawer.draw_text(
                    program,
                    Self::TEXT_PADDING,
                    y_padding,
                    self.config.font_size,
                    Color::BLACK,
                );
            } else {
                // Draw the program in white
                drawer.draw_text(
                    program,
                    Self::TEXT_PADDING,
                    y_padding,
                    self.config.font_size,
                    Color::WHITE,
                );
            }
        }
    }

    /// Draw the interface
    fn draw_interface(&mut self, drawer: &mut RaylibDrawHandle<'_>) {
        // Clear the background
        drawer.clear_background(Color::BLACK);

        // Draw the search bar background
        drawer.draw_rectangle(
            0,
            0,
            self.config.width,
            self.config.font_size + self.config.font_size / 2,
            Color::LIGHTGRAY,
        );

        // Draw the search bar
        drawer.draw_text(
            &self.search_bar,
            Self::TEXT_PADDING,
            self.config.font_size / 4,
            self.config.font_size,
            Color::BLACK,
        );

        // Draw the application list
        self.draw_application_list(drawer);
    }

    /// The main launcher for the program
    fn run(&mut self) {
        let (mut rl, thread) = raylib::init()
            .size(self.config.width, self.config.height)
            .vsync()
            .title(&self.config.title)
            .build();

        // The x and y coordinates for the FPS
        #[cfg(debug_assertions)]
        let fps_x = self.config.width - 50 - self.config.font_size * 2;
        #[cfg(debug_assertions)]
        let fps_y = self.config.width - self.config.font_size * 2;

        while !rl.window_should_close() {
            let pressed_key = rl.get_key_pressed();

            if self.handle_pressed_key(pressed_key) {
                break;
            }

            self.update_programs();

            // If the selected program is greater than the number of programs showing,
            // set it to the last program
            if self.selected_idx >= self.programs.len() && !self.programs.is_empty() {
                self.selected_idx = self.programs.len() - 1;
            }

            let mut d = rl.begin_drawing(&thread);
            self.draw_interface(&mut d);

            #[cfg(debug_assertions)]
            // Draw the FPS in debug mode
            d.draw_fps(fps_x, fps_y);
        }
    }

    fn update_programs(&mut self) {
        // Filter the programs based on the search bar
        self.programs = self
            .config
            .programs
            .iter()
            // TODO: implement some fuzy find algorithm
            .filter(|program| program.contains(&self.search_bar))
            .cloned()
            .collect();

        // Update the number of programs showing
        self.programs_count = self.programs.len().min(self.max);
    }

    /// Handle the pressed key and return a true if the main loop have to stop
    fn handle_pressed_key(&mut self, pressed_key: Option<KeyboardKey>) -> bool {
        match pressed_key {
            Some(raylib::consts::KeyboardKey::KEY_BACKSPACE) => {
                self.search_bar.pop();
            }
            Some(raylib::consts::KeyboardKey::KEY_ENTER) => {
                let selected_program = &self.programs[self.selected_idx];
                if Application::run_bash_command(&self.config, selected_program).is_ok() {
                    return true;
                }
            }
            Some(raylib::consts::KeyboardKey::KEY_DOWN) => {
                self.selected_idx += 1;
                if self.selected_idx >= self.programs_count {
                    self.selected_idx = 0;
                }
            }
            Some(raylib::consts::KeyboardKey::KEY_UP) => {
                if self.selected_idx == 0 {
                    self.selected_idx = self.programs_count - 1;
                } else {
                    self.selected_idx -= 1;
                }
            }
            Some(raylib::consts::KeyboardKey::KEY_ESCAPE) => {
                return true;
            }
            _ => unsafe {
                let mut key = GetCharPressed();

                while key > 0 {
                    self.search_bar.push(char::from_u32(key as u32).unwrap());
                    key = GetCharPressed();
                }
            },
        };

        false
    }
}

pub fn main() {
    const FONT_SIZE: i32 = 30;
    const WINDOW_WIDTH: i32 = 600;
    const WINDOW_HEIGHT: i32 = 800;

    let programs = Application::get_programs();

    let config = AppConfig {
        title: "Finsk".to_string(),
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
        font_size: FONT_SIZE,
        programs,
        launcher_program: "bash".to_string(),
    };

    let mut app = App::new(config);
    app.run();
}
