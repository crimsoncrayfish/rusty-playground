use rand::Rng;
use std::cmp::Ordering;
use std::io::{self, Write};

fn main() {
    println!("\x1b[2J\x1b[HLets guess something!"); //clear screen and move cursor to statr
    let mut line_count: i32 = 0;
    let mut previous_message: Option<String> = None;
    let mut prev_guess: Option<u32> = None;
    let mut has_result: bool = false;
    let mut has_err: bool = false;
    loop {
        while line_count > 0 {
            previous_line();
            line_count -= 1;
        }
        let secret_number = rand::thread_rng().gen_range(1..=100);

        let mut guess = String::new();

        if let Some(value) = previous_message {
            if has_result {
                println!("{}", value);
                line_count += 1;
                if let Some(prev_value) = prev_guess {
                    println!("You guessed {}", prev_value);
                    line_count += 1;
                }
            }
            if has_err {
                handle_err(value);
                line_count += 1;
            }
        }

        set_font_color("35");
        println!("The secret number is {secret_number}");
        reset_font_color();
        line_count += 1;

        //the words My geuss and the input will all be on one line
        print!("My guess: ");
        io::stdout().flush().unwrap();
        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");
        line_count += 1;

        if guess.trim() == "exit" {
            break;
        }

        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                previous_message = Some("Failed to convert string to int".to_string());
                has_result = false;
                has_err = true;
                continue;
            }
        };

        prev_guess = Some(guess);

        match guess.cmp(&secret_number) {
            Ordering::Less => {
                previous_message = Some("Too small!".to_string());
            }
            Ordering::Greater => {
                previous_message = Some("Too large!".to_string());
            }
            Ordering::Equal => {
                println!("Success!!!");
                break;
            }
        }
        has_err = false;
        has_result = true;
        //set forground red println!("\x1b[1;31m")
    }
}

fn handle_err(err_message: String) {
    set_font_color("31");
    println!("{}", err_message);
    reset_font_color();
}

fn set_font_color(color: &str) {
    print!("\x1b[1;{}m", color);
}

fn reset_font_color() {
    set_font_color("0");
}

fn previous_line() {
    print!("\x1b[1A\x1b[2K");
}
