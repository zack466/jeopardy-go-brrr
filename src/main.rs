use curl::easy::Easy;
use regex::Regex;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref RE_HTML: Regex =
        Regex::new(r#"(?:<(i|u|b)>|</(i|u|b)>|\&lt;(i|u|b)\&gt;|\&lt;/(i|u|b)\&gt;)"#).unwrap();
    static ref RE_CATEGORY: Regex = Regex::new(r#"<td class="category_name">(.+)</td>"#).unwrap();
    static ref RE_CLUE: Regex =
        Regex::new(r#"id="clue_J_(\d)_(\d)" class="clue_text">(.+)</td>"#).unwrap();
    static ref RE_ANSWER: Regex =
        Regex::new(r#"clue_J_(\d)_(\d).+correct_response\&quot;\&gt;(.+)\&lt;/em"#).unwrap();
    static ref RE_CLUE_D: Regex =
        Regex::new(r#"id="clue_DJ_(\d)_(\d)" class="clue_text">(.+)</td>"#).unwrap();
    static ref RE_ANSWER_D: Regex =
        Regex::new(r#"clue_DJ_(\d)_(\d).+correct_response\&quot;\&gt;(.+)\&lt;/em"#).unwrap();
    static ref RE_FINAL_CLUE: Regex =
        Regex::new(r#"id="clue_FJ" class="clue_text">(.+)</td>"#).unwrap();
    static ref RE_FINAL_ANSWER: Regex =
        Regex::new(r#"quot;correct_response\\&quot;\&gt;(.+)\&lt;/em"#).unwrap();
}

fn gen_url(game_id: usize) -> String {
    let base_url = "https://www.j-archive.com/showgame.php";
    format!("{}?game_id={}", base_url, game_id.to_string())
}

// TODO: fails if game not found
fn get_webpage(game_id: usize) -> String {
    let mut handle = Easy::new();
    let url = gen_url(game_id);
    handle.url(&url).unwrap();

    let mut buf = Vec::new();

    {
        let mut transfer = handle.transfer();
        transfer
            .write_function(|new_data| {
                buf.extend_from_slice(new_data);
                Ok(new_data.len())
            })
            .unwrap();
        transfer.perform().unwrap();
    }

    let s = match std::str::from_utf8(buf.as_slice()) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    return s.to_owned();
}

#[derive(Default, Clone, Debug)]
struct JeopardyQuestion {
    clue: String,
    answer: String,
    x: usize,     // num category from left to right
    y: usize,     // num question going down
    value: usize,
}

impl JeopardyQuestion {
    fn default() -> JeopardyQuestion {
        JeopardyQuestion {
            clue: String::new(),
            answer: String::new(),
            x: 0,
            y: 0,
            value: 0,
        }
    }
}

fn clean_html(data: String) -> String {
    let data = RE_HTML.replace_all(&data[..], "");
    let data = data.replace("&amp;", "&");
    let data = data.replace("<br />", "");
    return data;
}

fn populate_board(data: &String, board: &mut Board, double: bool) {
    // parse clues
    let re_clue: &Regex = if !double { &RE_CLUE } else { &RE_CLUE_D };
    for caps in re_clue.captures_iter(&data[..]) {
        let x: usize = caps.get(1).unwrap().as_str().parse().unwrap();
        let y: usize = caps.get(2).unwrap().as_str().parse().unwrap();
        let x = x - 1;
        let y = y - 1;
        let clue = caps.get(3).unwrap().as_str().to_string();
        let clue = clean_html(clue);
        board[y][x].x = x;
        board[y][x].y = y;
        board[y][x].clue.push_str(&clue);
        board[y][x].value = (y + 1) * 200;
        if double {
            board[y][x].value *= 2;
        }
    }

    // parse answers
    let re_answer: &Regex = if !double { &RE_ANSWER } else { &RE_ANSWER_D };
    for caps in re_answer.captures_iter(&data[..]) {
        let x: usize = caps.get(1).unwrap().as_str().parse().unwrap();
        let y: usize = caps.get(2).unwrap().as_str().parse().unwrap();
        let x = x - 1;
        let y = y - 1;
        let answer = caps.get(3).unwrap().as_str().to_string();
        let answer = clean_html(answer);
        board[y][x].answer.push_str(&answer);
    }
}

fn populate_categories(data: &String, categories: &mut Vec<Category>) {
    for caps in RE_CATEGORY.captures_iter(&data[..]) {
        categories.push(caps.get(1).unwrap().as_str().to_string());
    }
}

fn populate_final_jeopardy(data: &String, final_jeopardy: &mut JeopardyQuestion) {
    let clue = RE_FINAL_CLUE
        .captures(&data[..])
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
        .to_string();
    let answer = RE_FINAL_ANSWER
        .captures(&data[..])
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
        .to_string();
    let clue = clean_html(clue);
    let answer = clean_html(answer);
    final_jeopardy.clue = clue;
    final_jeopardy.answer = answer;
}

fn print_board(board: &Board) {
    for row in board {
        for jq in row {
            println!("{:?}", jq);
        }
    }
}

type Category = String;
type Board = Vec<Vec<JeopardyQuestion>>;

fn get_game_data(game_id: usize) -> (Vec<Category>, Board, Board, JeopardyQuestion) {
    let data = get_webpage(game_id);

    let mut board_1: Board = vec![vec![JeopardyQuestion::default(); 6]; 5];
    let mut board_2: Board = vec![vec![JeopardyQuestion::default(); 6]; 5];
    let mut categories = Vec::<Category>::new();
    let mut final_jeopardy = JeopardyQuestion::default();

    populate_categories(&data, &mut categories);
    populate_board(&data, &mut board_1, false);
    populate_board(&data, &mut board_2, true);
    populate_final_jeopardy(&data, &mut final_jeopardy);

    return (categories, board_1, board_2, final_jeopardy);
}

// TODO: move all this logic into lib.rs
// TODO: refactor for error handling
// TODO: add ability to cache j-archive pages, maybe just try and download every webpage
// TODO: ability to serialize data as JSON/database or some other format
// TODO: add ability to load from a local copy of webpage, a database, or cache
fn main() {
    let (categories, board_1, board_2, final_jeopardy) = get_game_data(7000);
    for c in categories {
        println!("{}", c);
    }
    print_board(&board_1);
    print_board(&board_2);
    println!("{:?}", final_jeopardy);

    // println!("{}", board_1[0][0]);
    // println!("{}", handle.response_code().unwrap());
    // println!("{}", gen_url(10));
}
