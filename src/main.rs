use std::io::{stdout, Write};
use std::time::Duration;

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor,
    Stylize, StyledContent},
    ExecutableCommand, Result,
    event, 
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, SetSize, size,
    disable_raw_mode, enable_raw_mode, Clear, ClearType::{All}},
    event::{read, Event, poll, KeyCode},
    cursor::{MoveUp, MoveDown, MoveLeft, MoveRight,
    MoveToColumn, MoveToRow, position, Hide, Show},
};

/*
    NOTES TO LUKE: 
        - All coords (column, row)
        - Bufs and Vec of columns, each rows number long

*/
fn main() -> Result<()> {
    enable_raw_mode()?;

    execute!(
        stdout(),
        EnterAlternateScreen,
        Hide,
        SetBackgroundColor(Color::Blue),
        SetForegroundColor(Color::Black),
    )?;

    let mut state = match State::new((80, 40)) { // columns, rows
        Ok(state) => state,
        Err(err) => panic!("{}", err),
    };

    state.draw_buffer_init()?;

    let mut x: i32 = 0;

    match read()? {
        _ => x += 1,
    }

    state.update_char_at_index((0, 0), 1);
    state.update_char_at_index((0, 1), 1);
    state.update_char_at_index((0, 2), 1);
    state.update_char_at_index((0, 3), 1);
    state.update_char_at_index((0, 4), 1);

    let matrix: Vec<Vec<u8>> = vec![
        vec![1, 1],
        vec![1, 1],
    ];

    state.insert_matrix_at_index((20, 20), matrix);
    state.update_use_buffer();

    state.draw_buffer_init()?;

    match read()? {
        _ => x += 1,
    }

    let (mut x, mut y) = (1, 0);
    let (mut old_x, mut old_y) = (0, 0);

    loop {
        state.update_char_at_index((x, y), 2);
        state.update_char_at_index((old_x, old_y), 0);

        state.draw_buffer()?;

        match is_event_availble() {
            Ok(true) => {
                match read()? {
                    Event::Key(key_event) => {
                        match key_event.code {
                            KeyCode::Char(c) => {
                                match c {
                                    'q' => break,
                                    _ => continue,
                                }
                            },
                            KeyCode::Right => {
                                old_x = x;
                                old_y = y;
                                x += 1;
                            },
                            KeyCode::Left => {
                                old_x = x;
                                old_y = y;
                                x -= 1;
                            },
                            KeyCode::Up => {
                                old_x = x;
                                old_y = y;
                                y -= 1;
                            },
                            KeyCode::Down => {
                                old_x = x;
                                old_y = y;
                                y += 1;
                            },
                            _ => continue,
                        }
                    },
                    _ => continue,
                }
            },
            _ => continue,
        }
    }

    disable_raw_mode()?;

    execute!(stdout(), ResetColor, LeaveAlternateScreen, Show)
}

fn is_event_availble() -> Result<bool> {
    poll(Duration::from_secs(0))
}

fn get_sub_view(buffer: &Vec<Vec<u8>>, location: (u16, u16), width: usize, height: usize) -> Vec<Vec<u8>> {
    let (column, row) : (usize, usize) = (location.0.into(), location.1.into());
    
    let view: Vec<Vec<u8>> = buffer[column..(column + width)]
                            .iter()
                            .map(|s| s[row..(row + height)].to_vec())
                            .collect();

    return view;
}

struct State {
    buffer: Vec<Vec<u8>>,
    alternate_buffer: Vec<Vec<u8>>,
    dimensions: (u16, u16), // columns, rows
    style_map: [StyledContent<char>; 4],
}

impl State {
    fn new(dimensions: (u16, u16)) -> Result<State> {
        execute!(
            stdout(),
            SetSize(dimensions.0, dimensions.1), // columns, rows
            Clear(All),
            MoveToColumn(0),
            MoveToRow(0),
        )?;

        let buffer = vec![vec![0; 40]; 80]; 
        // vec of colums, with rows number of elements
        let alternate_buffer = buffer.clone();
        let style_map = [' '.on_blue(), '.'.on_green(), '@'.on_blue(), '#'.on_blue()];

        Ok(State {
            buffer,
            alternate_buffer,
            dimensions,
            style_map,
        })
    }

    fn draw_buffer_init(&mut self) -> Result<()> {
        execute!(
            stdout(),
            Clear(All),
            MoveToColumn(0),
            MoveToRow(0),
        )?;
        
        let (columns, rows) : (usize, usize) = (self.dimensions.0.into(), self.dimensions.1.into());

        let mut view = get_sub_view(&self.alternate_buffer, (0, 0), columns, rows);

        for i in 0..rows {
            for column in &view {
                let character: StyledContent<char> = self.style_map[(column[i as usize]) as usize];
                execute!(stdout(), Print(character))?;
            }
        }

        Ok(())
    }

    fn draw_buffer(&mut self) -> Result<()> {
        // get current view window
        let (columns, rows) : (usize, usize) = (self.dimensions.0.into(), self.dimensions.1.into());

        let mut view: Vec<Vec<u8>> = get_sub_view(&self.alternate_buffer, (0, 0), columns, rows);
        let old_buffer: Vec<Vec<u8>> = get_sub_view(&self.buffer, (0, 0), columns, rows);
        
        // grab view size parts of current and old buffer
        // compare and print differnces
        for (x, column) in view.clone().iter().enumerate() {
            for (y, row) in column.iter().enumerate() {
                if *row == old_buffer[x][y] {
                    continue;
                } else {
                    let character: StyledContent<char> = self.style_map[old_buffer[x][y] as usize];
                    execute!(
                        stdout(),
                        MoveToColumn(x as u16),
                        MoveToRow(y as u16),
                        Print(character)
                    )?;
                }
            }
        }

        // update view buffer

        self.alternate_buffer = self.buffer.clone();

        execute!(
            stdout(),
            MoveToColumn(0),
            MoveToRow(0)
        )?;

        Ok(())
    }

    fn update_char_at_index(&mut self, location: (u16, u16), value: u8) {
        // use: (column, row) value
        let (column, row) : (usize, usize) = (location.0.into(), location.1.into());

        self.buffer[column][row] = value;
    }

    fn insert_matrix_at_index(&mut self, location: (u16, u16), matrix: Vec<Vec<u8>>) {
        let (column, row) : (usize, usize) = (location.0.into(), location.1.into());

        for (x, col) in matrix.iter().enumerate() {
            for (y, value) in col.iter().enumerate() {
                self.buffer[column + x][row + y] = *value;
            }
        }
    }

    fn update_use_buffer(&mut self) {
        self.alternate_buffer = self.buffer.clone();
    }
}