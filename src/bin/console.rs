use brrr::{self, JeopardyQuestion, Board};
use std::io;
use std::error::Error;
use tui::{Terminal};
use tui::backend::CrosstermBackend;
use tui::widgets::{Widget, Block, Borders, Table, Row, Cell};
use tui::layout::{Layout, Constraint, Direction};
use tui::style::{Style, Color, Modifier};
use crossterm;
use crossterm::{cursor, execute, terminal, event};
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use std::thread;
use std::sync::{self, mpsc};

type TERM = Terminal<CrosstermBackend<io::Stdout>>;

fn read_line() -> crossterm::Result<String> {
    let mut line = String::new();
    while let Event::Key(KeyEvent { code, .. }) = event::read()? {
        match code {
            KeyCode::Enter => {
                break;
            }
            KeyCode::Char(c) => {
                line.push(c);
            }
            _ => {}
        }
    }

    Ok(line)
}

fn game_selection() -> Result<usize, Box<dyn Error>> {
    println!("Type in a game id.");
    let input = read_line()?;
    let num = input.parse::<usize>()?;
    Ok(num)
}

struct GameState {
    answered: [[bool; 6]; 5],
    input_box: String
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            answered: [[false; 6]; 5],
            input_box: String::from("Hello there")
        }
    }
}

fn price_cell(x: usize, y: usize, double: bool) {

}

fn display_board(board: &Board, terminal: &mut TERM, state: &GameState) -> crossterm::Result<()> {
    terminal.draw(|f| {
        let size = f.size();
        let mut rows: Vec<Row> = vec![];
        for row in board {
            let mut r: Vec<Cell> = vec![];
            for question in row {
                let c = Cell::from(question.value().to_string());
                r.push(c);
            }
            rows.push(Row::new(r));
        }
        let input_block = Row::new(tui::text::Text::from(&state.input_box[..]));
        rows.push(input_block);
        let table = Table::new(rows)
            .style(Style::default().fg(Color::White))
            .block(Block::default().title("Board"))
            .widths(&[
                    Constraint::Percentage(16),
                    Constraint::Percentage(16),
                    Constraint::Percentage(16),
                    Constraint::Percentage(16),
                    Constraint::Percentage(16),
                    Constraint::Percentage(16),
            ])
            .column_spacing(1);
        f.render_widget(table, size);
    })?;
    Ok(())
}

fn play_board(categories: &[String], board: &Vec<Vec<JeopardyQuestion>>) {
    
}

fn play_final_jeopardy(category: &String, final_jeopardy: &JeopardyQuestion) {

}

fn mainloop(terminal: &mut TERM) -> Result<(), Box<dyn Error>> {
    let state = GameState::default();
    loop {
        let game_id = match game_selection() {
            Ok(x) => x,
            Err(_) => continue, // parse or input error
        };
        let game_data = match brrr::get_game_data(game_id) {
            Some(x) => x,
            None => continue, // game not found
        };

        let (all_categories, board_1, board_2, final_jeopardy) = game_data;

        display_board(&board_1, terminal, &state);
        break;

        play_board(&all_categories[..6], &board_1);

        play_board(&all_categories[6..12], &board_2);

        play_final_jeopardy(&all_categories[13], &final_jeopardy);

        break;
    }
    Ok(())

}


fn setup_terminal() -> crossterm::Result<TERM> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    crossterm::terminal::enable_raw_mode();
    terminal.clear()?;
    Ok(terminal)
}

fn stop_terminal(terminal: &mut TERM) -> crossterm::Result<()> {
    crossterm::terminal::disable_raw_mode()
}

fn main() {
    let mut terminal = setup_terminal().unwrap();

    match mainloop(&mut terminal) {
        Ok(()) => {},
        Err(e) => eprintln!("Error: {}", e),
    };
    stop_terminal(&mut terminal);
}
