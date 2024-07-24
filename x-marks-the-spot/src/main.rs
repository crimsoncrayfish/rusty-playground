use core::time;
use rand::Rng;
use std::{
    io::{self, Write},
    thread,
};

fn main() {
    hide_cursor();
    clear_screen();

    let height = 12;
    let width = 50;
    print_box(width, height);

    let mut rng = rand::thread_rng();
    let one_sec = time::Duration::from_millis(125);

    //player start
    let mut player_x = 2;
    let mut player_y = height - 1;
    let mut player_move_right = true;
    loop {
        let x = rng.gen_range(2..width - 1);
        let y = rng.gen_range(2..height - 1);
        x_marks_the_spot(x, y);

        let x = rng.gen_range(2..width - 1);
        let y = rng.gen_range(2..height - 1);
        x_unmarks_the_spot(x, y);
        (player_move_right, player_x, player_y) =
            place_player_bounce(player_move_right, player_x, player_y, width);

        let input = catch_input();
        if input == "ESC" {
            continue;
        }

        thread::sleep(one_sec);
    }
    //show_cursor();
}

fn place_player_bounce(
    mut direction_right: bool,
    mut old_x: i32,
    old_y: i32,
    width: i32,
) -> (bool, i32, i32) {
    clear_location(old_x, old_y);
    old_x = if direction_right {
        old_x + 1
    } else {
        old_x - 1
    };
    if old_x >= width - 1 {
        old_x = width - 2;
        direction_right = false;
    }
    if old_x <= 1 {
        old_x = 2;
        direction_right = true;
    }

    set_cursor_pos(old_x, old_y);
    set_fore_color("32".to_string());
    println!("A");
    (direction_right, old_x, old_y)
}

fn clear_location(x: i32, y: i32) {
    set_cursor_pos(x, y);
    println!(" ");
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
    set_back_color("47".to_string());
    set_fore_color("31".to_string());
    println!("X");
    set_back_color("0".to_string());
    set_fore_color("0".to_string());
}

fn x_unmarks_the_spot(x: i32, y: i32) {
    set_cursor_pos(x, y);
    set_fore_color("0".to_string());
    set_back_color("0".to_string());
    println!(" ");
}

fn set_fore_color(color: String) {
    print!("\x1b[1;{}m", color);
}

fn set_back_color(color: String) {
    print!("\x1b[1;{}m", color);
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
