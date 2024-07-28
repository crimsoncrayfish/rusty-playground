use rand::{rngs::StdRng, Rng, SeedableRng};
use std::{char, io::Write};

fn main() {
    clear_screen();
    let board_x: i32 = 10;
    let board_y: i32 = 10;
    let seed: u64 = 420;

    let mut gs = GameState::new(seed, board_x, board_y);
    gs.print_board();
    gs.print_state();

    /*    let seconds = Duration::from_secs(15);
    let sleep_ms = Duration::from_millis(200);
    //print board
    //set game
    //
    let start = SystemTime::now();
    loop {
        //calculate game state
        //update screen
        // let random_x: u16 = rng.gen_range(2..(board_x + 1));
        //   let random_y: u16 = rng.gen_range(2..(board_y + 1));
        //   print_char_to_location('X', random_x, random_y);
        //   let random_x: u16 = rng.gen_range(2..(board_x + 1));
        //   let random_y: u16 = rng.gen_range(2..(board_y + 1));
        //   print_char_to_location(' ', random_x, random_y);
        std::io::stdout()
            .flush()
            .expect("Could not flush the buffer");

        sleep(sleep_ms);
        match start.elapsed() {
            Ok(elapsed) if elapsed > seconds => {
                break;
            }
            _ => (),
        }
    }*/
    set_cursor_location(1, board_y + 3);
    println!("Done");
}

pub struct GameState {
    width: i32,
    height: i32,
    offset_x: i32,
    offset_y: i32,
    //a vec of length width * height to represent x and y coords
    previous: Vec<bool>,
    //a vec of length width * height to represent x and y coords
    next: Vec<bool>,
}

impl GameState {
    pub fn new(seed: u64, width: i32, height: i32) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);

        let mut previous: Vec<bool> = vec![false; (width * height) as usize];
        for i in 0..(width * height) - 1 {
            previous[i as usize] = rng.gen::<bool>()
        }
        let next: Vec<bool> = vec![false; (width * height) as usize];
        Self {
            width,
            offset_x: 1,
            offset_y: 1,
            height,
            previous,
            next,
        }
    }
}

impl GameState {
    pub fn xy_to_index(coordinate: Coord) -> i32 {
        coordinate.x * coordinate.y
    }
    pub fn index_to_xy(&mut self, index: i32) -> Coord {
        let index = index + 1;
        let x = index % self.width;
        let y = (index - (index % self.width)) / self.width;
        Coord { x, y }
    }
    pub fn print_board(&mut self) {
        //top && bottom
        for x in 2..self.width + 1 {
            print_char_to_location('-', x, 1);
            print_char_to_location('-', x, self.height + 2)
        }
        //sides
        for y in 2..self.height + 2 {
            print_char_to_location('|', 1, y);
            print_char_to_location('|', self.width + 2, y)
        }
        std::io::stdout()
            .flush()
            .expect("Could not flush the buffer");
    }

    pub fn print_state(&mut self) {
        for i in 0..(self.width * self.height) - 1 {
            let coord = self.index_to_xy(i);
            if self.previous[i as usize] {
                print_char_to_location(
                    'X',
                    coord.x + self.offset_x + 1,
                    coord.y + self.offset_y + 1,
                )
            } else {
                print_char_to_location(
                    ' ',
                    coord.x + self.offset_x + 1,
                    coord.y + self.offset_y + 1,
                )
            }
        }
        std::io::stdout()
            .flush()
            .expect("Could not flush the buffer");
    }
}

pub struct Coord {
    x: i32,
    y: i32,
}

fn clear_screen() {
    print!("\x1b[2J")
}

fn print_char_to_location(value: char, x: i32, y: i32) {
    set_cursor_location(x, y);
    print!("{value}");
}

fn set_cursor_location(x: i32, y: i32) {
    print!("\x1b[{y};{x}H");
}
