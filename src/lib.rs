use curl::easy::Easy;
use regex::Regex;
use std::fs::{self, File};
use std::io;
use std::io::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher,};

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

fn get_webpage(url: &String) -> String {
    let mut handle = Easy::new();
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

fn get_webpages(urls: Vec<&String>) -> Vec<String> {
    return vec![String::new()];
}

fn cache_read(url: &String) -> Option<String> {
    let filename = "./cache/".to_owned() + &url[47..];
    let data = fs::read_to_string(filename);
    match data {
        Ok(x) => Some(x),
        Err(e) => None,
    }
}

fn cache_write(url: &String, data: &String) {
    fs::create_dir_all("./cache");
    let filename = "./cache/".to_owned() + &url[47..];
    fs::write(filename, data).expect("Unable to write file");
}

#[derive(Default, Clone, Debug)]
pub struct JeopardyQuestion {
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
    pub fn value(&self) -> usize {
        return self.value;
    }
    pub fn clue(&self) -> &str {
        return &self.clue[..];
    }
    pub fn answer(&self) -> &str {
        return &self.answer[..];
    }
}

fn clean_html(data: String) -> String {
    let data = RE_HTML.replace_all(&data[..], "");
    let data = data.replace("&amp;", "&");
    let data = data.replace("<br />", "");
    let data = data.replace("\\'", "'");
    let data = data.replace("\\\"", "\"");
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
        let category = caps.get(1).unwrap().as_str().to_string();
        let category = clean_html(category);
        categories.push(category);
    }
}

fn populate_final_jeopardy(data: &String, final_jeopardy: &mut JeopardyQuestion) {
    for caps in RE_FINAL_CLUE.captures_iter(&data) {
        let clue = caps.get(1).unwrap().as_str();
        let clue = clean_html(clue.to_string());
        final_jeopardy.clue = clue;
    }
    for caps in RE_FINAL_ANSWER.captures_iter(&data) {
        let answer = caps.get(1).unwrap().as_str();
        let answer = clean_html(answer.to_string());
        final_jeopardy.answer = answer;
    }
}

pub fn print_board(board: &Board) {
    for row in board {
        for jq in row {
            println!("{:?}", jq);
        }
    }
}

pub type Category = String;
pub type Board = Vec<Vec<JeopardyQuestion>>;
pub type Game = (Vec<Category>, Board, Board, JeopardyQuestion);

pub fn get_game_data(game_id: usize) -> Option<Game> {
    // missing in j-archive
    if [1132].contains(&game_id) {
        return None;
    }

    let url = gen_url(game_id);
    let data = match cache_read(&url) {
        Some(data) => {
            // println!("Loading {} from cache...", game_id);
            data
        },
        None => {
            // respectful web scraping etiquette
            // std::thread::sleep(std::time::Duration::from_secs(20));

            // println!("Loading {} from j-archive...", game_id);
            let data = get_webpage(&url);
            // if game not in online database
            if data.contains("ERROR") {
                return None;
            } else {
                data
            }
        }
    };

    cache_write(&url, &data);

    let mut board_1: Board = vec![vec![JeopardyQuestion::default(); 6]; 5];
    let mut board_2: Board = vec![vec![JeopardyQuestion::default(); 6]; 5];
    let mut categories = Vec::<Category>::new();
    let mut final_jeopardy = JeopardyQuestion::default();

    populate_categories(&data, &mut categories);
    populate_board(&data, &mut board_1, false);
    populate_board(&data, &mut board_2, true);
    populate_final_jeopardy(&data, &mut final_jeopardy);

    return Some((categories, board_1, board_2, final_jeopardy));
}

// TODO: refactor for error handling
// TODO: ability to serialize data as JSON/database or some other format
