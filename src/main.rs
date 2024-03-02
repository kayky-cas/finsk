use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use raylib::{
    ffi::{GetCharPressed, IsKeyDown, LoadFontFromMemory, Vector2},
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
    programs: Vec<&'static str>,
    launcher_program: String,
    font_path: &'static [u8],
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

        let font = self.load_font();

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
        let mut programs_filtered: Vec<_> = self
            .config
            .programs
            .iter()
            // TODO: Change that to my own algorithm
            .map(|program| (program, matcher.fuzzy_match(program, &self.search_bar)))
            .filter(|program| program.1.is_some())
            .collect();

        programs_filtered.sort_by(|(_, score_a), (_, score_b)| score_a.cmp(score_b));

        self.programs = programs_filtered
            .into_iter()
            .map(|(p, _)| p)
            .cloned()
            .collect();

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
            // Using unsafe because GetCharPressed is a C function
            Some(_) => unsafe {
                let mut key = GetCharPressed();

                while key > 0 {
                    self.search_bar.push(char::from_u32(key as u32).unwrap());
                    key = GetCharPressed();
                }
            },
            _ => {}
        };

        false
    }

    fn load_font(&self) -> Font {
        // I don't like this but ok

        let font_data = self.config.font_path;
        let font_size = self.config.font_path.len();
        let font_ft = CString::new(".ttf").unwrap();
        let chars = null_mut();

        unsafe {
            Font::from_raw(LoadFontFromMemory(
                font_ft.as_ptr(),
                font_data.as_ptr(),
                font_size.try_into().unwrap(),
                self.config.font_size,
                chars,
                100,
            ))
        }
    }
}

pub fn main() {
    const FONT_SIZE: i32 = 32;
    const WINDOW_WIDTH: i32 = 500;
    const WINDOW_HEIGHT: i32 = 800;

    let programs = Application::get_programs()
        .iter()
        .map(|p| p.clone().leak() as &'static str)
        .collect();

    let config = AppConfig {
        title: "Finsk".to_string(),
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
        font_size: FONT_SIZE,
        programs,
        launcher_program: "bash".to_string(),
        font_path: include_bytes!("../resources/Roboto-Regular.ttf"),
    };

    let mut app = App::new(config);
    app.run();
}
