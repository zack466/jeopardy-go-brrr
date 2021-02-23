use std::io::{stdout, Write};
use curl::easy::Easy;
use regex::Regex;

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

fn main() {
    let re = Regex::new(r"battle (\w+)").unwrap();
    let data = get_webpage(100);
    for caps in re.captures_iter(&data[..]) {
        println!("{}", caps.get(1).unwrap().as_str());
    }

    // println!("{}", handle.response_code().unwrap());
    // println!("{}", gen_url(10));
}

