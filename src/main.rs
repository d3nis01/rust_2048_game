use colored::*;
use crossterm::{
    cursor,
    event::{read, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    ExecutableCommand,
};
use rand::{seq::IteratorRandom, thread_rng, Rng};
use std::collections::HashMap;
use std::io::{stdout, Write};

fn main() -> crossterm::Result<()> {
    enable_raw_mode()?;
    let mut game_board = vec![vec![0; 4]; 4];
    let mut colors: HashMap<u32, Color> = HashMap::new();
    let mut current_score = 0;

    initialize_colors(&mut colors);
    spawn_random_tile(&mut game_board);
    spawn_random_tile(&mut game_board);
    render_board(&game_board, &colors, current_score)?;

    loop {
        if let Event::Key(key_event) = read()? {
            match key_event.code {
                KeyCode::Char('e') | KeyCode::Char('E') => break,
                _ => {
                    let moved = match key_event.code {
                        KeyCode::Up => move_up(&mut game_board),
                        KeyCode::Down => move_down(&mut game_board),
                        KeyCode::Left => move_left(&mut game_board),
                        KeyCode::Right => move_right(&mut game_board),
                        _ => false,
                    };

                    if moved {
                        spawn_random_tile(&mut game_board);
                        current_score = calculate_score(&game_board);
                        render_board(&game_board, &colors, current_score)?;
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
) -> crossterm::Result<()> {
    let mut stdout: std::io::Stdout = stdout();
    stdout.execute(Clear(ClearType::All))?;
    stdout.execute(cursor::MoveTo(0, 0))?;

    for row in game_board {
        for &val in row {
            let color = colors.get(&val).unwrap_or(&Color::White);
            write!(stdout, "{} ", format!("{:4}", val).color(*color))?;
        }
        writeln!(stdout)?;
    }
    writeln!(stdout, " > Current score : {}", current_score)?;
    writeln!(stdout)?;
    writeln!(stdout, " > Press E to exit")?;
    stdout.flush()?;
    Ok(())
}

fn calculate_score(game_board: &[Vec<u32>]) -> u32 {
    game_board.iter().flatten().sum()
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
    let mut moved = false;
    for row in game_board.iter_mut() {
        for i in 1..row.len() {
            if row[i] != 0 && row[i - 1] == 0 {
                row.swap(i, i - 1);
                moved = true;
            }
        }

        for i in 1..row.len() {
            if row[i] != 0 && row[i] == row[i - 1] {
                row[i - 1] *= 2;
                row[i] = 0;
                moved = true;
            }
        }

        for i in 1..row.len() {
            if row[i] != 0 && row[i - 1] == 0 {
                row.swap(i, i - 1);
            }
        }
    }
    moved
}

fn move_right(game_board: &mut [Vec<u32>]) -> bool {
    let mut moved = false;
    for row in game_board.iter_mut() {
        for i in (0..row.len() - 1).rev() {
            if row[i] != 0 && row[i + 1] == 0 {
                row.swap(i, i + 1);
                moved = true;
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
            if row[i] != 0 && row[i + 1] == 0 {
                row.swap(i, i + 1);
            }
        }
    }
    moved
}

fn move_up(game_board: &mut [Vec<u32>]) -> bool {
    let mut moved = false;
    for col in 0..game_board[0].len() {
        for row in 1..game_board.len() {
            if game_board[row][col] != 0 && game_board[row - 1][col] == 0 {
                game_board[row - 1][col] = game_board[row][col];
                game_board[row][col] = 0;
                moved = true;
            }
        }

        for row in 1..game_board.len() {
            if game_board[row][col] != 0 && game_board[row][col] == game_board[row - 1][col] {
                game_board[row - 1][col] *= 2;
                game_board[row][col] = 0;
                moved = true;
            }
        }

        for row in 1..game_board.len() {
            if game_board[row][col] != 0 && game_board[row - 1][col] == 0 {
                game_board[row - 1][col] = game_board[row][col];
                game_board[row][col] = 0;
            }
        }
    }
    moved
}

fn move_down(game_board: &mut [Vec<u32>]) -> bool {
    let mut moved = false;
    for col in 0..game_board[0].len() {
        for row in (0..game_board.len() - 1).rev() {
            if game_board[row][col] != 0 && game_board[row + 1][col] == 0 {
                game_board[row + 1][col] = game_board[row][col];
                game_board[row][col] = 0;
                moved = true;
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
            if game_board[row][col] != 0 && game_board[row + 1][col] == 0 {
                game_board[row + 1][col] = game_board[row][col];
                game_board[row][col] = 0;
            }
        }
    }
    moved
}
