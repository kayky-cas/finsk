use raylib::{ffi::GetCharPressed, prelude::*};
use std::{
    collections::HashSet,
    io,
    process::{Child, Command},
};

#[derive(Default)]
struct AppState {
    title: String,
    width: i32,
    height: i32,
    font_size: i32,
    programs: Vec<String>,
    launcher_program: String,
}

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
fn run_bash_command(state: &AppState, command: &str) -> io::Result<Child> {
    Command::new(&state.launcher_program)
        .args(["-c", command])
        .spawn()
}

/// Draw the application list
fn draw_application_list(
    drawer: &mut RaylibDrawHandle<'_>,
    state: &AppState,
    programs: &[String],
    max: usize,
    selected_program: usize,
    padding: i32,
) {
    for (idx, program) in programs.iter().take(max).enumerate() {
        let y_padding = (idx as i32 * state.font_size) + state.font_size * 2;

        if idx == selected_program {
            // Draw the ligtgray background for the selected program
            drawer.draw_rectangle(0, y_padding, state.width, state.font_size, Color::LIGHTGRAY);
            // Draw the selected program in black
            drawer.draw_text(program, padding, y_padding, state.font_size, Color::BLACK);
        } else {
            // Draw the program in white
            drawer.draw_text(program, padding, y_padding, state.font_size, Color::WHITE);
        }
    }
}

/// Draw the interface
fn draw_interface(
    drawer: &mut RaylibDrawHandle<'_>,
    state: &AppState,
    search_bar: &str,
    programs: &[String],
    max: usize,
    selected_program: usize,
) {
    // Clear the background
    drawer.clear_background(Color::BLACK);

    // Draw the search bar background
    drawer.draw_rectangle(
        0,
        0,
        state.width,
        state.font_size + state.font_size / 2,
        Color::LIGHTGRAY,
    );

    const TEXT_PADDING: i32 = 10;

    // Draw the search bar
    drawer.draw_text(
        search_bar,
        TEXT_PADDING,
        state.font_size / 4,
        state.font_size,
        Color::BLACK,
    );

    // Draw the application list
    draw_application_list(drawer, state, programs, max, selected_program, TEXT_PADDING);
}

/// The main launcher for the program
fn launcher(state: AppState) {
    let mut current_programs = state.programs.clone();

    let (mut rl, thread) = raylib::init()
        .size(state.width, state.height)
        .vsync()
        .title(&state.title)
        .build();

    let mut search_bar = String::new();
    let mut selected_program = 0;

    // The maximum number of programs showing
    let programs_max = ((state.height / state.font_size) - 2) as usize;

    // The number of programs showing
    let mut programs_current_showing = current_programs.len().min(programs_max);

    // The x and y coordinates for the FPS
    #[cfg(debug_assertions)]
    let fps_x = state.width - 50 - state.font_size * 2;
    #[cfg(debug_assertions)]
    let fps_y = state.width - state.font_size * 2;

    'runner: while !rl.window_should_close() {
        let pressed_key = rl.get_key_pressed();

        match pressed_key {
            Some(raylib::consts::KeyboardKey::KEY_BACKSPACE) => {
                search_bar.pop();
            }
            Some(raylib::consts::KeyboardKey::KEY_ENTER) => {
                if run_bash_command(&state, &current_programs[selected_program]).is_ok() {
                    break 'runner;
                }
            }
            Some(raylib::consts::KeyboardKey::KEY_DOWN) => {
                selected_program += 1;
                if selected_program >= programs_current_showing {
                    selected_program = 0;
                }
            }
            Some(raylib::consts::KeyboardKey::KEY_UP) => {
                if selected_program == 0 {
                    selected_program = programs_current_showing - 1;
                } else {
                    selected_program -= 1;
                }
            }
            Some(raylib::consts::KeyboardKey::KEY_ESCAPE) => {
                break 'runner;
            }
            _ => unsafe {
                let mut key = GetCharPressed();

                while key > 0 {
                    search_bar.push(char::from_u32(key as u32).unwrap());
                    key = GetCharPressed();
                }
            },
        }

        // Filter the programs based on the search bar
        current_programs = state
            .programs
            .iter()
            .filter(|program| program.contains(&search_bar))
            .cloned()
            .collect();

        // Update the number of programs showing
        programs_current_showing = current_programs.len().min(programs_max);

        // If the selected program is greater than the number of programs showing,
        // set it to the last program
        if selected_program >= current_programs.len() && !current_programs.is_empty() {
            selected_program = current_programs.len() - 1;
        }

        let mut d = rl.begin_drawing(&thread);

        draw_interface(
            &mut d,
            &state,
            &search_bar,
            &current_programs,
            programs_max,
            selected_program,
        );

        #[cfg(debug_assertions)]
        // Draw the FPS in debug mode
        d.draw_fps(fps_x, fps_y);
    }
}

pub fn main() {
    const FONT_SIZE: i32 = 30;
    const WINDOW_WIDTH: i32 = 600;
    const WINDOW_HEIGHT: i32 = 800;

    let programs = get_programs();

    let state = AppState {
        title: "Finsk".to_string(),
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
        font_size: FONT_SIZE,
        programs,
        launcher_program: "bash".to_string(),
    };

    launcher(state);
}
