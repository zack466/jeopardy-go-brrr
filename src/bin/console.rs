use brrr::{self, Board, JeopardyQuestion};
use crossterm;
use crossterm::event::{read, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use crossterm::{cursor, event, execute, terminal};
use std::{error::Error, thread::sleep, time::Duration};
use std::io;
use std::sync::{self, mpsc};
use std::thread;
use tui::backend::CrosstermBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Wrap, Block, Clear, Borders, Cell, Paragraph, Row, Table, Tabs, Widget};
use tui::{symbols::DOT, Terminal};

type TERM = Terminal<CrosstermBackend<io::Stdout>>;

fn game_selection(prompt: &str, terminal: &mut TERM, key_rx: &mpsc::Receiver<KeyEvent>) -> Result<Option<usize>, Box<dyn Error>> {
    let mut input = String::new();
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .constraints([
                             Constraint::Percentage(20),
                             Constraint::Percentage(60),
                             Constraint::Percentage(20),
                ].as_ref())
                .direction(Direction::Horizontal)
                .split(f.size());
            {
                let chunks = Layout::default()
                    .constraints([
                                 Constraint::Percentage(30),
                                 Constraint::Percentage(40),
                                 Constraint::Percentage(30),
                    ].as_ref())
                    .split(chunks[1]);
                let prompt = Paragraph::new(vec![
                                            Spans::from(Span::from("Enter game id:")),
                                            Spans::from(Span::from(prompt)),
                                            Spans::from(Span::from(&input[..]))
                ])
                    .block(Block::default().borders(Borders::ALL))
                    .alignment(Alignment::Center);
                f.render_widget(prompt, chunks[1]);
            }
        })?;
        if let Ok(event) = key_rx.recv() {
            match event.code {
                KeyCode::Char('q') => {
                    return Ok(None);
                },
                KeyCode::Enter => {
                    break;
                }
                KeyCode::Backspace => {
                    if input.len() > 0 {
                        input.pop();
                    }
                }
                KeyCode::Char(x) => {
                    input.push(x);
                }
                _ => {}
            }
        }
    }
    let num = input.parse::<usize>()?;
    Ok(Some(num))
}

struct Coords {
    x: usize,
    y: usize,
}

struct GameState {
    answered: [[bool; 6]; 5],
    input_box: String,
    selected: Coords,
}

impl GameState {
    fn up(&mut self) {
        self.selected.y -= if self.selected.y > 0 { 1 } else { 0 };
    }
    fn down(&mut self) {
        self.selected.y += if self.selected.y < 4 { 1 } else { 0 };
    }
    fn left(&mut self) {
        self.selected.x -= if self.selected.x > 0 { 1 } else { 0 };
    }
    fn right(&mut self) {
        self.selected.x += if self.selected.x < 5 { 1 } else { 0 };
    }
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            answered: [[false; 6]; 5],
            input_box: String::from("Hello there"),
            selected: Coords { x: 0, y: 0 },
        }
    }
}

fn display_board(
    categories: &[String],
    board: &Board,
    terminal: &mut TERM,
    state: &GameState,
) -> crossterm::Result<()> {
    terminal.draw(|f| {
        let chunks = Layout::default()
            .constraints(
                [
                    Constraint::Percentage(16),
                    Constraint::Percentage(16),
                    Constraint::Percentage(16),
                    Constraint::Percentage(16),
                    Constraint::Percentage(16),
                    Constraint::Percentage(16),
                ]
                .as_ref(),
            )
            .direction(Direction::Horizontal)
            .split(f.size());

        for i in 0..6 {
            let chunks = Layout::default()
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(10),
                        Constraint::Percentage(18),
                        Constraint::Percentage(18),
                        Constraint::Percentage(18),
                        Constraint::Percentage(18),
                        Constraint::Percentage(18),
                    ]
                    .as_ref(),
                )
                .split(chunks[i]);
            let title = (&categories[i]).to_owned();
            let title = Paragraph::new(title)
                .block(Block::default().borders(Borders::ALL))
                .style(Style::default().add_modifier(Modifier::BOLD))
                .wrap(Wrap {trim: false})
                .alignment(Alignment::Center);
            f.render_widget(title, chunks[0]);
            for j in 0..5 {
                // prints $value if question is valid and not answered
                let text = if !state.answered[j][i] && board[j][i].value() != 0 {
                    Spans::from(vec![Span::from(format!(
                        "${}",
                        board[j][i].value().to_string()
                    ))])
                } else {
                    Spans::from(vec![Span::from("")])
                };
                let text = if state.selected.x == i && state.selected.y == j {
                    Paragraph::new(text)
                        .wrap(Wrap {trim: false})
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .border_style(Style::default().fg(Color::Blue)),
                        )
                        .alignment(Alignment::Center)
                } else {
                    Paragraph::new(text)
                        .wrap(Wrap {trim: false})
                        .block(Block::default().borders(Borders::ALL))
                        .alignment(Alignment::Center)
                };
                f.render_widget(text, chunks[j + 1]);
            }
        }
    })?;
    Ok(())
}

fn render_textbox(text: &str, terminal: &mut TERM) -> crossterm::Result<()> {
    terminal.draw(move |f| {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(15), Constraint::Percentage(70), Constraint::Percentage(15)].as_ref())
            .split(f.size());
        {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(15), Constraint::Percentage(70), Constraint::Percentage(15)].as_ref())
                .split(chunks[1]);
            let text = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL))
                .wrap(Wrap {trim: false})
                .alignment(Alignment::Center);
            f.render_widget(Clear, chunks[1]);
            f.render_widget(text, chunks[1]);
        }
    })?;
    Ok(())
}

fn display_clue(question: &JeopardyQuestion, terminal: &mut TERM, state: &mut GameState, key_rx: &mpsc::Receiver<KeyEvent>) -> crossterm::Result<GameResult> {
    loop {
        render_textbox(question.clue(), terminal)?;
        if let Ok(event) = key_rx.recv() {
            match event.code {
                KeyCode::Enter => {
                    state.answered[state.selected.y][state.selected.x] = true;
                    break;
                },
                KeyCode::Char('q') => {
                    break;
                },
                _ => { }
            }
        }
    }
    loop {
        render_textbox(question.answer(), terminal)?;
        if let Ok(event) = key_rx.recv() {
            match event.code {
                KeyCode::Enter => {
                    break;
                },
                KeyCode::Char('q') => {
                    return Ok(GameResult::Quit);
                },
                KeyCode::Delete => {
                    break;
                }
                _ => { }
            }
        }
    }
    return Ok(GameResult::Continue);
}

enum GameResult {
    Continue,
    Quit,
}

fn play_board(
    categories: &[String],
    board: &Vec<Vec<JeopardyQuestion>>,
    terminal: &mut TERM,
    state: &mut GameState,
    key_rx: &mpsc::Receiver<KeyEvent>,
) -> crossterm::Result<GameResult> {
    loop {
        display_board(categories, &board, terminal, &state)?;
        if let Ok(event) = key_rx.recv() {
            match event.code {
                KeyCode::Enter => {
                    match display_clue(&board[state.selected.y][state.selected.x], terminal, state, key_rx) {
                        Ok(GameResult::Continue) => {},
                        Ok(GameResult::Quit) => return Ok(GameResult::Quit),
                        Err(e) => return Err(e),
                    }
                },
                KeyCode::Char('q') => {
                    return Ok(GameResult::Quit);
                },
                KeyCode::Up => {
                    state.up();
                },
                KeyCode::Down => {
                    state.down();
                },
                KeyCode::Left => {
                    state.left();
                },
                KeyCode::Right => {
                    state.right();
                },
                KeyCode::Char(' ') => {
                    break;
                }
                _ => { }
            }
        }
    }
    Ok(GameResult::Continue)
}

fn play_final_jeopardy(category: &String, final_jeopardy: &JeopardyQuestion) {}

fn mainloop(terminal: &mut TERM, key_rx: &mpsc::Receiver<KeyEvent>) -> Result<(), Box<dyn Error>> {
    let mut state = GameState::default();
    let mut msg = "";
    loop {
        let game_id = match game_selection(msg, terminal, key_rx) {
            Ok(Some(x)) => x,
            Ok(None) => break,
            Err(_) => {
                msg = "Invalid game id";
                continue;
            }, // parse or input error
        };
        let game_data = match brrr::get_game_data(game_id) {
            Some(x) => x,
            None => {
                msg = "Game not found";
                continue;
            }, // game not found
        };

        let (all_categories, board_1, board_2, final_jeopardy) = game_data;

        match play_board(&all_categories[..6], &board_1, terminal, &mut state, &key_rx) {
            Ok(GameResult::Continue) => {
                // continues
            },
            Ok(GameResult::Quit) => {
                break;
            },
            _ => break,
        }

        match play_board(&all_categories[6..12], &board_2, terminal, &mut state, &key_rx) {
            Ok(GameResult::Continue) => {
                // continues
            },
            Ok(GameResult::Quit) => {
                break;
            },
            _ => break,
        }

        play_final_jeopardy(&all_categories[12], &final_jeopardy);
    }
    Ok(())
}

fn setup_input() -> crossterm::Result<mpsc::Receiver<KeyEvent>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || loop {
        if event::poll(Duration::from_millis(100)).unwrap() {
            if let Ok(Event::Key(event)) = read() {
                tx.send(event);
            }
        }
    });
    Ok(rx)
}

fn setup_terminal() -> crossterm::Result<TERM> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    crossterm::terminal::enable_raw_mode()?;
    terminal.clear()?;
    Ok(terminal)
}

fn stop_terminal(terminal: &mut TERM) -> crossterm::Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

fn main() {
    let mut terminal = setup_terminal().unwrap();

    let key_rx = setup_input().unwrap();

    match mainloop(&mut terminal, &key_rx) {
        Ok(()) => {}
        Err(e) => eprintln!("Error: {}", e),
    };
    stop_terminal(&mut terminal);
}
