use colored::*;
use crossterm::{
    cursor,
    event::{read, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand,
};
use rand::{seq::IteratorRandom, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, stdout, Write},
};

#[derive(Serialize, Deserialize)]
struct GameState {
    game_board: Vec<Vec<u32>>,
    current_score: u32,
    high_score: u32,
}

fn save_game_state(state: &GameState) -> Result<(), Box<dyn std::error::Error>> {
    let serialized = serde_json::to_string(state)?;
    fs::write("game_state.json", serialized)?;
    Ok(())
}

fn load_game_state() -> Option<GameState> {
    let data = fs::read_to_string("game_state.json").ok()?;
    serde_json::from_str(&data).ok()
}

fn main() -> crossterm::Result<()> {
    enable_raw_mode()?;
    let mut colors: HashMap<u32, Color> = HashMap::new();
    let mut state = load_game_state().unwrap_or_else(|| GameState {
        game_board: vec![vec![0; 4]; 4],
        current_score: 0,
        high_score: read_high_score(),
    });
    state.current_score = calculate_score(&state.game_board);

    let mut high_score = read_high_score();
    initialize_colors(&mut colors);
    if state.current_score == 0 {
        spawn_random_tile(&mut state.game_board);
        spawn_random_tile(&mut state.game_board);
    }
    render_board(&state.game_board, &colors, state.current_score, high_score)?;

    loop {
        if let Event::Key(key_event) = read()? {
            match key_event.code {
                KeyCode::Char('e') | KeyCode::Char('E') => {
                    if let Err(e) = save_game_state(&state) {
                        eprintln!(" > Failed to save game state: {}", e);
                    }
                    break;
                }
                _ => {
                    let moved: bool = match key_event.code {
                        KeyCode::Up => move_up(&mut state.game_board),
                        KeyCode::Down => move_down(&mut state.game_board),
                        KeyCode::Left => move_left(&mut state.game_board),
                        KeyCode::Right => move_right(&mut state.game_board),
                        _ => false,
                    };

                    if moved {
                        spawn_random_tile(&mut state.game_board);
                        state.current_score = calculate_score(&state.game_board);

                        if state.current_score > high_score {
                            high_score = state.current_score;
                            if let Err(e) = write_high_score(high_score) {
                                eprintln!(" > Failed to write high score: {}", e);
                            }
                        }

                        if !can_make_move(&state.game_board) {
                            render_board(
                                &state.game_board,
                                &colors,
                                state.current_score,
                                high_score,
                            )?;
                            let start_state: GameState = GameState {
                                game_board: vec![vec![0; 4]; 4],
                                current_score: 0,
                                high_score: read_high_score(),
                            };

                            if let Err(e) = save_game_state(&start_state) {
                                eprintln!(" > Failed to save game state: {}", e);
                            }

                            println!(" >> Game Over! <<");
                            break;
                        }

                        render_board(&state.game_board, &colors, state.current_score, high_score)?;
                    }
                }
            }
        }
    }
    disable_raw_mode()?;
    Ok(())
}

fn render_board(
    game_board: &Vec<Vec<u32>>,
    colors: &HashMap<u32, Color>,
    current_score: u32,
    high_score: u32,
) -> crossterm::Result<()> {
    let mut stdout: std::io::Stdout = stdout();
    stdout.execute(Clear(ClearType::All))?;
    stdout.execute(cursor::MoveTo(0, 0))?;

    for row in game_board {
        for &val in row {
            let color = colors.get(&val).unwrap_or(&Color::White);
            print!("{} ", format!("{:4}", val).color(*color));
        }
        println!();
    }
    println!(" > Current score : {}", current_score);
    println!(" > High score    : {}", high_score);
    println!();
    println!(" > Press E to exit");
    Ok(())
}

fn can_make_move(game_board: &[Vec<u32>]) -> bool {
    for row in game_board {
        for i in 0..row.len() {
            if row[i] == 0 {
                return true;
            }
            if i < row.len() - 1 && row[i] == row[i + 1] {
                return true;
            }
        }
    }

    for col in 0..game_board[0].len() {
        for row in 0..game_board.len() - 1 {
            if game_board[row][col] == game_board[row + 1][col] {
                return true;
            }
        }
    }

    false
}

fn calculate_score(game_board: &[Vec<u32>]) -> u32 {
    game_board.iter().flatten().sum()
}

fn read_high_score() -> u32 {
    let file_path = "highscore.txt";
    match fs::read_to_string(file_path) {
        Ok(content) => content.trim().parse().unwrap_or(0),
        Err(_) => 0,
    }
}

fn write_high_score(high_score: u32) -> io::Result<()> {
    let mut file = File::create("highscore.txt")?;
    write!(file, "{}", high_score)?;
    Ok(())
}

fn initialize_colors(colors: &mut HashMap<u32, Color>) {
    colors.insert(2, Color::Red);
    colors.insert(4, Color::Blue);
    colors.insert(8, Color::Green);
    colors.insert(16, Color::Yellow);
    colors.insert(32, Color::Magenta);
    colors.insert(64, Color::Cyan);
    colors.insert(128, Color::BrightRed);
    colors.insert(256, Color::BrightBlue);
    colors.insert(512, Color::BrightGreen);
    colors.insert(1024, Color::Green);
    colors.insert(2048, Color::BrightCyan);
}

fn spawn_random_tile(game_board: &mut [Vec<u32>]) {
    let mut empty_cells_array: Vec<(usize, usize)> = Vec::new();
    for (i, row) in game_board.iter().enumerate() {
        for (j, &cell) in row.iter().enumerate() {
            if cell == 0 {
                empty_cells_array.push((i, j));
            }
        }
    }

    if let Some(&(i, j)) = empty_cells_array.iter().choose(&mut thread_rng()) {
        let new_value = if thread_rng().gen_bool(0.9) { 2 } else { 4 };
        game_board[i][j] = new_value;
    }
}

fn move_left(game_board: &mut [Vec<u32>]) -> bool {
    let initial_board = game_board.to_vec();
    let mut moved = false;

    for row in game_board.iter_mut() {
        for i in 1..row.len() {
            let mut k = i;
            while k > 0 && row[k - 1] == 0 {
                row.swap(k, k - 1);
                moved = true;
                k -= 1;
            }
        }
        for i in 0..row.len() - 1 {
            if row[i] != 0 && row[i] == row[i + 1] {
                row[i] *= 2;
                row[i + 1] = 0;
                moved = true;
            }
        }
        for i in 1..row.len() {
            let mut k = i;
            while k > 0 && row[k - 1] == 0 {
                row.swap(k, k - 1);
                k -= 1;
            }
        }
    }

    if initial_board == *game_board {
        moved = false;
    }

    moved
}

fn move_right(game_board: &mut [Vec<u32>]) -> bool {
    let mut moved = false;
    let initial_board = game_board.to_vec();

    for row in game_board.iter_mut() {
        for i in (0..row.len() - 1).rev() {
            let mut k = i;
            while k < row.len() - 1 && row[k + 1] == 0 {
                row.swap(k, k + 1);
                moved = true;
                k += 1;
            }
        }

        for i in (0..row.len() - 1).rev() {
            if row[i] != 0 && row[i] == row[i + 1] {
                row[i + 1] *= 2;
                row[i] = 0;
                moved = true;
            }
        }
        for i in (0..row.len() - 1).rev() {
            let mut k = i;
            while k < row.len() - 1 && row[k + 1] == 0 {
                row.swap(k, k + 1);
                k += 1;
            }
        }
    }

    if initial_board == *game_board {
        moved = false;
    }

    moved
}

fn move_up(game_board: &mut [Vec<u32>]) -> bool {
    let mut moved = false;
    let initial_board = game_board.to_vec();

    for col in 0..game_board[0].len() {
        for row in 1..game_board.len() {
            let mut k = row;
            while k > 0 && game_board[k - 1][col] == 0 {
                game_board[k - 1][col] = game_board[k][col];
                game_board[k][col] = 0;
                moved = true;
                k -= 1;
            }
        }
        for row in 0..game_board.len() - 1 {
            if game_board[row][col] != 0 && game_board[row][col] == game_board[row + 1][col] {
                game_board[row][col] *= 2;
                game_board[row + 1][col] = 0;
                moved = true;
            }
        }
        for row in 1..game_board.len() {
            let mut k = row;
            while k > 0 && game_board[k - 1][col] == 0 {
                game_board[k - 1][col] = game_board[k][col];
                game_board[k][col] = 0;
                k -= 1;
            }
        }
    }

    if initial_board == *game_board {
        moved = false;
    }

    moved
}

fn move_down(game_board: &mut [Vec<u32>]) -> bool {
    let mut moved = false;
    let initial_board = game_board.to_vec();

    for col in 0..game_board[0].len() {
        for row in (0..game_board.len() - 1).rev() {
            let mut k = row;
            while k < game_board.len() - 1 && game_board[k + 1][col] == 0 {
                game_board[k + 1][col] = game_board[k][col];
                game_board[k][col] = 0;
                moved = true;
                k += 1;
            }
        }
        for row in (0..game_board.len() - 1).rev() {
            if game_board[row][col] != 0 && game_board[row][col] == game_board[row + 1][col] {
                game_board[row + 1][col] *= 2;
                game_board[row][col] = 0;
                moved = true;
            }
        }
        for row in (0..game_board.len() - 1).rev() {
            let mut k = row;
            while k < game_board.len() - 1 && game_board[k + 1][col] == 0 {
                game_board[k + 1][col] = game_board[k][col];
                game_board[k][col] = 0;
                k += 1;
            }
        }
    }

    if initial_board == *game_board {
        moved = false;
    }

    moved
}
