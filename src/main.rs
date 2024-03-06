use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use raylib::{
    ffi::{GetCharPressed, LoadFontFromMemory, Vector2},
    prelude::*,
};
use std::{
    collections::HashSet,
    ffi::CString,
    io,
    process::{Child, Command},
    ptr::null_mut,
};

struct Application;

impl Application {
    /// Get all the programs in the system (and leanking the strings... oops)
    fn get_programs() -> Vec<&'static str> {
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

        programs
            .into_iter()
            .map(|program| program.leak() as &'static str)
            .collect()
    }

    /// Run a command for the launcher_program
    fn run_bash_command(state: &AppConfig, command: &str) -> io::Result<Child> {
        Command::new(&state.launcher_program)
            .args(["-c", command])
            .spawn()
    }
}

/// Wrapper to the unsafe part of raylib
struct Bridge;

impl Bridge {
    fn get_key_pressed() -> char {
        let key = unsafe { GetCharPressed() as u32 };
        char::from_u32(key).unwrap()
    }

    fn load_font_from_memory(font_data: &[u8], font_size: i32, font_file_type: &str) -> Font {
        let font_ft = CString::new(font_file_type).unwrap();
        unsafe {
            Font::from_raw(LoadFontFromMemory(
                font_ft.as_ptr(),
                font_data.as_ptr(),
                font_data.len().try_into().unwrap(),
                font_size,
                null_mut(),
                100,
            ))
        }
    }
}

/// A struct that defines the initial configuration of the app.
/// The use of `i32` is because is raylib default's
#[derive(Default)]
struct AppConfig {
    title: String,
    width: i32,
    height: i32,
    font_size: i32,
    programs: Vec<&'static str>,
    launcher_program: String,
    font_data: &'static [u8],
    font_file_type: &'static str,
}

struct Position {
    x: i32,
    y: i32,
}

impl From<Position> for Vector2 {
    fn from(val: Position) -> Self {
        Vector2 {
            x: val.x as f32,
            y: val.y as f32,
        }
    }
}

struct App {
    config: AppConfig,
    selected_idx: usize,
    programs: Vec<&'static str>,
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
    fn draw_application_list(&mut self, drawer: &mut RaylibDrawHandle<'_>, font: &Font) {
        for (idx, program) in self.programs.iter().take(self.max).enumerate() {
            let text_position = Position {
                x: Self::TEXT_PADDING,
                y: (idx as i32 * self.config.font_size) + self.config.font_size * 2,
            };

            let text_color = if idx == self.selected_idx {
                // Draw the ligtgray background for the selected program
                drawer.draw_rectangle(
                    0i32,
                    text_position.y,
                    self.config.width,
                    self.config.font_size,
                    Color::LIGHTGRAY,
                );
                Color::BLACK
            } else {
                Color::WHITE
            };

            // Draw the program in the text_color
            drawer.draw_text_ex(
                font,
                program,
                text_position,
                self.config.font_size as f32,
                0.0f32,
                text_color,
            );
        }
    }

    /// Draw the interface
    fn draw_interface(&mut self, drawer: &mut RaylibDrawHandle<'_>, font: &Font) {
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

        let text_position = Position {
            x: Self::TEXT_PADDING,
            y: self.config.font_size / 4,
        };

        // Draw the search bar
        drawer.draw_text_ex(
            font,
            &self.search_bar,
            text_position,
            self.config.font_size as f32,
            0.0f32,
            Color::BLACK,
        );

        // Draw the application list
        self.draw_application_list(drawer, font);
    }

    /// The main launcher for the program
    fn run(&mut self) {
        let matcher = SkimMatcherV2::default();

        let (mut rl, thread) = raylib::init()
            .size(self.config.width, self.config.height)
            .vsync()
            .title(&self.config.title)
            .msaa_4x()
            .build();

        // Load font from memory
        let font = Bridge::load_font_from_memory(
            self.config.font_data,
            self.config.font_size,
            self.config.font_file_type,
        );

        // The x and y coordinates for the FPS
        #[cfg(debug_assertions)]
        let fps_position = Position {
            x: self.config.width - 50 - self.config.font_size * 2,
            y: self.config.width - self.config.font_size * 2,
        };

        while !rl.window_should_close() {
            let pressed_key = rl.get_key_pressed();

            if self.stop_from_pressed_key(pressed_key) {
                break;
            }

            self.update_programs(&matcher);

            // If the selected program is greater than the number of programs showing,
            // set it to the last program
            if self.selected_idx >= self.programs.len() && !self.programs.is_empty() {
                self.selected_idx = self.programs.len() - 1;
            }

            let mut d = rl.begin_drawing(&thread);
            self.draw_interface(&mut d, &font);

            #[cfg(debug_assertions)]
            // Draw the FPS in debug mode
            d.draw_fps(fps_position.x, fps_position.y);
        }
    }

    fn update_programs(&mut self, matcher: &dyn FuzzyMatcher) {
        // Filter the programs based on the search bar
        let mut programs_filtered: Vec<&'static str> = self
            .config
            .programs
            .iter()
            // TODO: Change that to my own algorithm
            .flat_map(|&program| {
                matcher
                    .fuzzy_match(program, &self.search_bar)
                    .map(|_| program)
            })
            .collect();

        programs_filtered.sort_by_key(|program| program.len());

        self.programs = programs_filtered;

        // Update the number of programs showing
        self.programs_count = self.programs.len().min(self.max);
    }

    /// Handle the pressed key and return a true if the main loop have to stop
    fn stop_from_pressed_key(&mut self, pressed_key: Option<KeyboardKey>) -> bool {
        match pressed_key {
            Some(KeyboardKey::KEY_BACKSPACE) => {
                self.search_bar.pop();
            }
            Some(KeyboardKey::KEY_ENTER) => {
                let selected_program = &self.programs[self.selected_idx];
                if Application::run_bash_command(&self.config, selected_program).is_ok() {
                    return true;
                }
            }
            Some(KeyboardKey::KEY_DOWN) => {
                self.selected_idx += 1;
                if self.selected_idx >= self.programs_count {
                    self.selected_idx = 0;
                }
            }
            Some(KeyboardKey::KEY_UP) => {
                if self.selected_idx == 0 {
                    self.selected_idx = self.programs_count - 1;
                } else {
                    self.selected_idx -= 1;
                }
            }
            Some(KeyboardKey::KEY_ESCAPE) => {
                return true;
            }
            Some(_) => {
                let mut ch = Bridge::get_key_pressed();

                while ch > '\0' {
                    self.search_bar.push(ch);
                    ch = Bridge::get_key_pressed();
                }
            }
            _ => {}
        };

        false
    }
}

pub fn main() {
    const FONT_SIZE: i32 = 32;
    const WINDOW_WIDTH: i32 = 500;
    const WINDOW_HEIGHT: i32 = 800;

    let config = AppConfig {
        title: "Finsk".to_string(),
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
        font_size: FONT_SIZE,
        programs: Application::get_programs(),
        launcher_program: "zsh".to_string(),
        font_data: include_bytes!("../resources/JetBrainsMonoNLNerdFontMono-Regular.ttf"),
        font_file_type: ".ttf",
    };

    let mut app = App::new(config);
    app.run();
}
