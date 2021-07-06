use std::io::{stdout, Write};

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor,
    Stylize, StyledContent},
    ExecutableCommand, Result,
    event,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, SetSize, size,
    disable_raw_mode, enable_raw_mode, Clear, ClearType::{All}},
    event::{read, Event},
    cursor::{MoveUp, MoveDown, MoveLeft, MoveRight,
    MoveToColumn, MoveToRow, position},
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
        SetBackgroundColor(Color::Blue),
        SetForegroundColor(Color::Black),
    )?;

    let mut state = match State::new((80, 40)) { // columns, rows
        Ok(state) => state,
        Err(err) => panic!("{}", err),
    };

    state.draw_buffer()?;

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

    state.draw_buffer()?;

    match read()? {
        _ => x += 1,
    }

    disable_raw_mode()?;

    execute!(stdout(), ResetColor, LeaveAlternateScreen)
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

    fn draw_buffer(&mut self) -> Result<()> {
        execute!(
            stdout(),
            Clear(All),
            MoveToColumn(0),
            MoveToRow(0),
        )?;

        let mut view: Vec<Vec<u8>> = Vec::new();

        let (columns, rows) : (usize, usize) = (self.dimensions.0.into(), self.dimensions.1.into());

        let iter = self.alternate_buffer[0..columns]
                    .iter()
                    .map(|s| &s[0..rows]);

        for column in iter {
            view.push((*column).to_vec());
        }

        for i in 0..rows {
            for column in &view {
                let character: StyledContent<char> = self.style_map[(column[i as usize]) as usize];
                execute!(stdout(), Print(character))?;
            }
        }

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