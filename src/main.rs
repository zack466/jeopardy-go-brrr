use curl::easy::Easy;
use regex::Regex;

#[macro_use]
extern crate lazy_static;

fn gen_url(game_id: usize) -> String {
    let base_url = "https://www.j-archive.com/showgame.php";
    format!("{}?game_id={}", base_url, game_id.to_string())
}

fn get_webpage(game_id: usize) -> String {
    let mut handle = Easy::new();
    let url = gen_url(game_id);
    handle.url(&url).unwrap();

    let mut buf = Vec::new();

    {
        let mut transfer = handle.transfer();
        transfer.write_function(|new_data| {
            buf.extend_from_slice(new_data);
            Ok(new_data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }

    let s = match std::str::from_utf8(buf.as_slice()) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    return s.to_owned();
}

lazy_static! {
    static ref RE_HTML: Regex = Regex::new(r#"(?:<(i|u|b)>|</(i|u|b)>|\&lt;(i|u|b)\&gt;|\&lt;/(i|u|b)\&gt;)"#).unwrap();
    static ref RE_CATEGORY: Regex = Regex::new(r#"<td class="category_name">(.+)</td>"#).unwrap();
    static ref RE_CLUE: Regex = Regex::new(r#"id="clue_J_(\d)_(\d)" class="clue_text">(.+)</td>"#).unwrap();
    static ref RE_ANSWER: Regex = Regex::new(r#"clue_J_(\d)_(\d).+correct_response\&quot;\&gt;(.+)\&lt;/em"#).unwrap();
    static ref RE_CLUE_D: Regex = Regex::new(r#"id="clue_DJ_(\d)_(\d)" class="clue_text">(.+)</td>"#).unwrap();
    static ref RE_ANSWER_D: Regex = Regex::new(r#"clue_DJ_(\d)_(\d).+correct_response\&quot;\&gt;(.+)\&lt;/em"#).unwrap();
}

//TODO: move all HTML cleaning into a single function
fn clean_html(data: String) -> String {
    return data;
}

#[derive(Default, Clone, Debug)]
struct JeopardyQuestion {
    clue: String,
    answer: String,
    x: usize, // num category from left to right
    y: usize, // num question going down
    double: bool, // second round is doubled
}

impl JeopardyQuestion {
    fn value(&self) -> usize {
        return (self.y + 1) * 200 * if self.double { 2 } else { 1 };
    }
    fn default() -> JeopardyQuestion {
        JeopardyQuestion {
            clue: String::new(),
            answer: String::new(),
            x: 0,
            y: 0,
            double: false,
        }
    }
}

fn populate_board(data: &String, board: &mut Board, double: bool) {
    // parse Jeopardy clues
    let re_clue: &Regex = if !double {&RE_CLUE} else {&RE_CLUE_D};
    for caps in re_clue.captures_iter(&data[..]) {
        let x: usize = caps.get(1).unwrap().as_str().parse().unwrap();
        let y: usize = caps.get(2).unwrap().as_str().parse().unwrap();
        let x = x - 1;
        let y = y - 1;
        let clue = caps.get(3).unwrap().as_str().to_string();
        let clue = RE_HTML.replace_all(&clue[..], "");
        let clue = clue.replace("&amp", "&");//.replace("\\\"", "\"").replace("\\\'", "\'");
        board[y][x].x = x;
        board[y][x].y = y;
        board[y][x].clue.push_str(&clue);
        if double {
            board[y][x].double = true;
        }
    }

    // parse correct answers
    let re_answer: &Regex = if !double {&RE_ANSWER} else {&RE_ANSWER_D};
    for caps in re_answer.captures_iter(&data[..]) {
        let x: usize = caps.get(1).unwrap().as_str().parse().unwrap();
        let y: usize = caps.get(2).unwrap().as_str().parse().unwrap();
        let x = x - 1;
        let y = y - 1;
        let answer = caps.get(3).unwrap().as_str().to_string();
        let answer = RE_HTML.replace_all(&answer[..], "");
        let answer = answer.replace("&amp;", "&").replace("\\\"", "\"").replace("\\\'", "\'");
        board[y][x].answer.push_str(&answer);
    }
}

fn populate_categories(data: &String, categories: &mut Vec<Category>) {
    for caps in RE_CATEGORY.captures_iter(&data[..]) {
        categories.push(caps.get(1).unwrap().as_str().to_string());
    }
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

// TODO: parse final jeopardy

fn get_game_data(game_id: usize) -> (Vec<Category>, Board, Board) {
    let data = get_webpage(game_id);

    let mut board_1: Board = vec![vec![JeopardyQuestion::default(); 6]; 5];
    let mut board_2: Board = vec![vec![JeopardyQuestion::default(); 6]; 5];
    let mut categories = Vec::<Category>::new();

    populate_categories(&data, &mut categories);
    populate_board(&data, &mut board_1, false);
    populate_board(&data, &mut board_2, true);

    return (categories, board_1, board_2);
}

fn main() {
    let (categories, board_1, board_2) = get_game_data(6000);
    for c in categories {
        println!("{}", c);
    }
    print_board(&board_1);
    print_board(&board_2);

    // println!("{}", board_1[0][0]);
    // println!("{}", handle.response_code().unwrap());
    // println!("{}", gen_url(10));
}

