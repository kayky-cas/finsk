use raylib::{ffi::GetCharPressed, prelude::*};
use std::{collections::HashSet, process::Command};

#[derive(Default)]
struct AppState {
    title: String,
    width: i32,
    height: i32,
    font_size: i32,
}

fn get_programs() -> Vec<String> {
    // Get all the programs in the system
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

fn launcher(state: AppState) {
    let programs = get_programs();

    let mut current_programs = programs.clone();

    let (mut rl, thread) = raylib::init()
        .size(state.width, state.height)
        .vsync()
        .title(&state.title)
        .build();

    let mut search_bar = String::new();

    let mut selected_program = 0;

    let programs_max = ((state.height / state.font_size) - 2) as usize;
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
                if Command::new("bash")
                    .args(["-c", &current_programs[selected_program]])
                    .spawn()
                    .is_ok()
                {
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
        current_programs = programs
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

        d.clear_background(Color::BLACK);

        // Draw the search bar background
        d.draw_rectangle(
            0,
            0,
            state.width,
            state.font_size + state.font_size / 2,
            Color::LIGHTGRAY,
        );

        const TEXT_PADDING: i32 = 10;

        // Draw the search bar
        d.draw_text(
            &search_bar,
            TEXT_PADDING,
            state.font_size / 4,
            state.font_size,
            Color::BLACK,
        );

        for (idx, program) in current_programs.iter().take(programs_max).enumerate() {
            let y_padding = (idx as i32 * state.font_size) + state.font_size * 2;

            if idx == selected_program {
                // Draw the ligtgray background for the selected program
                d.draw_rectangle(0, y_padding, state.width, state.font_size, Color::LIGHTGRAY);
                // Draw the selected program in black
                d.draw_text(
                    program,
                    TEXT_PADDING,
                    y_padding,
                    state.font_size,
                    Color::BLACK,
                );
            } else {
                // Draw the program in white
                d.draw_text(
                    program,
                    TEXT_PADDING,
                    y_padding,
                    state.font_size,
                    Color::WHITE,
                );
            }
        }

        #[cfg(debug_assertions)]
        // Draw the FPS in debug mode
        d.draw_fps(fps_x, fps_y);
    }
}

pub fn main() {
    const FONT_SIZE: i32 = 30;
    const WINDOW_WIDTH: i32 = 600;
    const WINDOW_HEIGHT: i32 = 800;

    let state = AppState {
        title: "Finsk".to_string(),
        width: WINDOW_WIDTH,
        height: WINDOW_HEIGHT,
        font_size: FONT_SIZE,
    };

    launcher(state);
}
