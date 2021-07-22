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

pub type Dimemsion = (u16, u16);
pub type Buffer = Vec<Vec<u8>>;
pub type StyleMap = Vec<StyledContent<char>>;

fn get_sub_view(buffer: &Buffer, location: (u16, u16), width: usize, height: usize) -> Buffer {
    let (column, row) : (usize, usize) = (location.0.into(), location.1.into());
    
    let view: Buffer = buffer[column..(column + width)]
                            .iter()
                            .map(|s| s[row..(row + height)].to_vec())
                            .collect();

    return view;
}

pub struct Renderer {
    pub input_buffer: Buffer,
    pub render_buffer: Buffer,
    pub dimensions: Dimemsion,
    pub view_port: Dimemsion,
    pub style_map: StyleMap,
}

impl Renderer {
    pub fn initialize(dimensions: Dimemsion, view_port: Dimemsion, style_map: StyleMap) -> Result<Renderer> {
        enable_raw_mode()?;
        execute!(
        stdout(), EnterAlternateScreen, Hide,
        SetBackgroundColor(Color::Black),
        SetForegroundColor(Color::Black),
        )?;
        
        execute!(
            stdout(),
            SetSize(view_port.0, view_port.1), // columns, rows
            Clear(All),
            MoveToColumn(0),
            MoveToRow(0),
        )?;

        let buffer = vec![vec![0; dimensions.1 as usize + 1]; dimensions.0 as usize + 1];

        Ok(Renderer{
            input_buffer: buffer.clone(),
            render_buffer: buffer.clone(),
            dimensions,
            view_port,
            style_map,
        })

    }

    pub fn reset_term() -> Result<()> {
        disable_raw_mode()?;
        execute!(stdout(), ResetColor, LeaveAlternateScreen, Show)?;
        Ok(())
    }

    pub fn render(&mut self) -> Result<()> {
        // find dimensions
        let view_box: (usize, usize) = ((self.view_port.0 + 1).into(), (self.view_port.1 + 1).into());

        let current_render: Buffer = get_sub_view(&self.render_buffer, (0, 0), view_box.0, view_box.1);
        let modified: Buffer = get_sub_view(&self.input_buffer, (0, 0), view_box.0, view_box.1);

        // find modified points and draw them
        for (x, column) in current_render.iter().enumerate() {
            for (y, row) in column.iter().enumerate() {
                if *row == modified[x][y] {
                    continue;
                } else {
                    let character: StyledContent<char> = self.style_map[modified[x][y] as usize];
                    execute!(
                        stdout(),
                        MoveToColumn(x as u16),
                        MoveToRow(y as u16),
                        Print(character)
                    )?;
                }
            }
        }

        // upadte render buffer, then different now the same
        self.render_buffer = self.input_buffer.clone();
        // reset cursor
        execute!(
            stdout(),
            MoveToColumn(0),
            MoveToRow(0)
        )?;

        Ok(())
    }

    pub fn insert_char(&mut self, location: (u16, u16), value: u8) {
        let single_matrix = vec![vec![value]];

        self.insert_matrix(location, single_matrix);
    }

    pub fn insert_matrix(&mut self, location: (u16, u16), matrix: Buffer) {
        let (column, row) : (usize, usize) = (location.0.into(), location.1.into());

        for (x, col) in matrix.iter().enumerate() {
            for (y, value) in col.iter().enumerate() {
                self.input_buffer[column + x][row + y] = *value;
            }
        }
    }
}