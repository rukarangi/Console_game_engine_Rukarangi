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

pub fn get_sub_view(buffer: &Vec<Vec<u8>>, location: (u16, u16), width: usize, height: usize) -> Vec<Vec<u8>> {
    let (column, row) : (usize, usize) = (location.0.into(), location.1.into());
    
    let view: Vec<Vec<u8>> = buffer[column..(column + width)]
                            .iter()
                            .map(|s| s[row..(row + height)].to_vec())
                            .collect();

    return view;
}

pub struct State {
    pub buffer: Vec<Vec<u8>>,
    pub alternate_buffer: Vec<Vec<u8>>,
    pub dimensions: (u16, u16), // columns, rows
    pub style_map: [StyledContent<char>; 7],
}

impl State {
    pub fn new(dimensions: (u16, u16)) -> Result<State> {
        execute!(
            stdout(),
            SetSize(dimensions.0, dimensions.1), // columns, rows
            Clear(All),
            MoveToColumn(0),
            MoveToRow(0),
        )?;

        let buffer = vec![vec![0; 100]; 100]; 
        // vec of colums, with rows number of elements
        let alternate_buffer = buffer.clone();
        let style_map = [
            ' '.on_blue(),  // 0
            '.'.on_green(), // 1
            '@'.on_blue(),  // 2
            '#'.on_blue(),  // 3
            '|'.on_green(), // 4
            ' '.on_green(), // 5
            '_'.on_green(), // 6
        ];

        Ok(State {
            buffer,
            alternate_buffer,
            dimensions,
            style_map,
        })
    }

    pub fn draw_buffer_init(&mut self) -> Result<()> {
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

    pub fn draw_buffer(&mut self) -> Result<()> {
        // get current view window
        let (columns, rows) : (usize, usize) = ((self.dimensions.0 + 1).into(), (self.dimensions.1 + 1).into());

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

    pub fn update_char_at_index(&mut self, location: (u16, u16), value: u8) {
        // use: (column, row) value
        let (column, row) : (usize, usize) = (location.0.into(), location.1.into());

        self.buffer[column][row] = value;
    }

    pub fn insert_matrix_at_index(&mut self, location: (u16, u16), matrix: Vec<Vec<u8>>) {
        let (column, row) : (usize, usize) = (location.0.into(), location.1.into());

        for (x, col) in matrix.iter().enumerate() {
            for (y, value) in col.iter().enumerate() {
                self.buffer[column + x][row + y] = *value;
            }
        }
    }

    pub fn update_use_buffer(&mut self) {
        self.alternate_buffer = self.buffer.clone();
    }
}