use core::time;
use rand::Rng;
use std::{
    io::{self, Write},
    thread,
};

fn main() {
    let width = 50;
    let height = 12;
    clear_screen();
    print_box(width, height);
    let mut rng = rand::thread_rng();
    let one_sec = time::Duration::from_secs(1);
    hide_cursor();
    loop {
        let x = rng.gen_range(2..width - 1);
        let y = rng.gen_range(2..height - 1);
        x_marks_the_spot(x, y);
        thread::sleep(one_sec);
        let input = catch_input();
        if input == "ESC" {
            continue;
        }
    }
    show_cursor();
}

fn catch_input() -> std::string::String {
    "".to_string()
}

fn show_cursor() {
    println!("\x1b[?25h")
}
fn hide_cursor() {
    println!("\x1b[?25l")
}

fn x_marks_the_spot(x: i32, y: i32) {
    set_cursor_pos(x, y);
    println!("X")
}

fn print_box(width: i32, height: i32) {
    print_horizontal_line(0, width); //this is row 1
    for y in 2..height {
        set_cursor_pos(0, y);
        clear_line();
        print_left_and_right_box(y, width);
    }
    print_horizontal_line(height, width);
}

fn print_horizontal_line(start_y: i32, width: i32) {
    let mut start_string = String::new();
    for _ in 1..width {
        start_string += "-"
    }
    print_line_num(0, start_y, start_string)
}

fn print_left_and_right_box(y: i32, width: i32) {
    set_cursor_pos(0, y);
    print_wall();
    set_cursor_pos(width - 1, y);
    print_wall();
    io::stdout().flush().expect("Failed to flush output")
}

fn print_wall() {
    print!("|")
}

fn print_line_num(x: i32, y: i32, text: String) {
    set_cursor_pos(x, y);
    clear_line();
    println!("{}", text);
}

fn set_cursor_pos(x: i32, y: i32) {
    print!("\x1b[{};{}H", y, x);
}

fn clear_line() {
    print!("\x1b[2K")
}

fn clear_screen() {
    println!("\x1b[2J");
}
