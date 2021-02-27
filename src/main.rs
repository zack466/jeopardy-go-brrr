use brrr::{get_game_data, print_board};
use std::env::args;

fn cache_games(x: usize, y: usize) {
    for game_id in x..y {
        let game = match get_game_data(game_id) {
            Some(x) => x,
            None => {
                println!("Game {} could not be loaded.", game_id);
                continue;
            }
        };
    }
}

// TODO: parse args and add options
fn main() {
    let a: Vec<String> = args().collect();
    let game_id: usize = a[1].parse().unwrap();

    let game = match get_game_data(game_id) {
        Some(x) => x,
        None => {
            println!("Game {} could not be loaded.", game_id);
            return;
        }
    };

    let (categories, board_1, board_2, final_jeopardy) = game;

    println!("Categories: ");
    for c in categories {
        print!("{} ", c);
    }
    println!("");

    println!("First round: ");
    print_board(&board_1);
    println!("Second round: ");
    print_board(&board_2);
    println!("Final jeopardy: ");
    println!("{:?}", final_jeopardy);
}
