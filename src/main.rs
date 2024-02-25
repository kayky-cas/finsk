use raylib::{ffi::GetCharPressed, prelude::*};
use std::{collections::HashSet, process::Command};

const FONT_SIZE: i32 = 30;

pub fn main() {
    let output = Command::new("bash")
        .args(["-c", "compgen -c"])
        .output()
        .unwrap()
        .stdout;

    let bindings = String::from_utf8(output).unwrap();
    let programs: HashSet<&str> = bindings.lines().collect();
    let programs: Vec<&str> = programs.into_iter().collect();

    let mut current_programs = programs.clone();

    let (mut rl, thread) = raylib::init().size(600, 800).title("Finsk").build();

    let mut search_bar = String::from("");

    let mut selected_program = 0;

    unsafe {
        while !rl.window_should_close() {
            let pressed_key = rl.get_key_pressed();
            let mut d = rl.begin_drawing(&thread);

            d.clear_background(Color::BLACK);

            d.draw_rectangle(0, 0, 600, FONT_SIZE + 10, Color::LIGHTGRAY);
            d.draw_text(&search_bar, 10, 5, FONT_SIZE, Color::BLACK);

            for (i, program) in current_programs.iter().enumerate() {
                let y = (i as i32 * FONT_SIZE) + 50;

                if i == selected_program {
                    d.draw_rectangle(0, y, 600, FONT_SIZE, Color::LIGHTGRAY);
                    d.draw_text(program, 10, y, FONT_SIZE, Color::BLACK);
                } else {
                    d.draw_text(program, 10, y, FONT_SIZE, Color::WHITE);
                }
            }

            let mut key = GetCharPressed();

            while key > 0 {
                search_bar.push(char::from_u32(key as u32).unwrap());
                key = GetCharPressed();
            }

            match pressed_key {
                Some(raylib::consts::KeyboardKey::KEY_BACKSPACE) => {
                    search_bar.pop();
                }
                Some(raylib::consts::KeyboardKey::KEY_ENTER) => {
                    Command::new("bash")
                        .args(["-c", &current_programs[selected_program]])
                        .spawn()
                        .unwrap();

                    break;
                }
                Some(raylib::consts::KeyboardKey::KEY_DOWN) => {
                    selected_program += 1;
                    if selected_program >= current_programs.len() {
                        selected_program = 0;
                    }
                }
                Some(raylib::consts::KeyboardKey::KEY_UP) => {
                    selected_program = selected_program.wrapping_sub(1);
                    if selected_program >= current_programs.len() {
                        selected_program = current_programs.len() - 1;
                    }
                }
                Some(raylib::consts::KeyboardKey::KEY_ESCAPE) => {
                    break;
                }
                _ => {}
            }

            current_programs = programs
                .iter()
                .filter(|program| program.contains(&search_bar))
                .copied()
                .collect();

            if selected_program >= current_programs.len() && !current_programs.is_empty() {
                selected_program = current_programs.len() - 1;
            }
        }
    }
}
