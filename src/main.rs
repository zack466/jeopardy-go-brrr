use brrr::{get_game_data, print_board};
use std::env::args;

fn cache_all(x: usize, y: usize) {
    for game_id in x..y {
        let game = match get_game_data(game_id) {
            Some(x) => x,
            None => {
                println!("Game {} could not be loaded.", game_id);
                continue;
            },
        };
    }
}

// options:
// - 
fn main() {
    let a: Vec<String> = args().collect();
    let x: usize = a[1].parse().unwrap();
    let y: usize = a[2].parse().unwrap();
    cache_all(x, y);
    // let game_id = 3000;
    // let game = match get_game_data(game_id) {
    //     Some(x) => x,
    //     None => {
    //         println!("Game {} could not be loaded.", game_id);
    //         return;
    //     },
    // };

    // let (categories, board_1, board_2, final_jeopardy) = game;
    // for c in categories {
    //     println!("{}", c);
    // }
    // print_board(&board_1);
    // print_board(&board_2);
    // println!("{:?}", final_jeopardy);

    // println!("{}", board_1[0][0]);
    // println!("{}", handle.response_code().unwrap());
    // println!("{}", gen_url(10));
}
