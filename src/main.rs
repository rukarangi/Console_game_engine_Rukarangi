use std::io::{stdout, Write};

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor,
    Stylize, StyledContent},
    ExecutableCommand, Result,
    event,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, SetSize, size,
    disable_raw_mode, enable_raw_mode},
    event::{read, Event},
    cursor::{MoveUp, MoveDown, MoveLeft, MoveRight,
    MoveToColumn, MoveToRow, position},
};

//const U8TOCHAR: [StyledContent<char>; 4] = [' '.on_blue(), '.'.on_green(), '@'.on_blue(), '#'.on_blue()];

fn deconstruct_buffer(buf: Vec<Vec<u8>>) {
    let u8ToChar: [StyledContent<char>; 4] = [' '.on_blue(), '.'.on_green(), '@'.on_blue(), '#'.on_blue()];

    //let mut result = String::new();
    
    for row in buf.iter() {
        for c in row.iter() {
            execute!(stdout(), Print(u8ToChar[*c as usize]));
        }
    }
}

fn main() -> Result<()> {
    let (cols, rows) = size()?;

    let mut buf: Vec<Vec<u8>> = vec![vec![0; rows.into()]; cols.into()];

    let mut new_buf: Vec<Vec<u8>> = Vec::new(); //vec![vec![0; rows.into()]; cols.into()];

    for (index, row) in buf.iter().enumerate() {
        new_buf.push(vec![]);
        for (idx, c) in row.iter().enumerate() {
            if index & 2 == 0 {
                new_buf[index].push(0);
            } else {
                new_buf[index].push(1);
            }
        }
    }

    enable_raw_mode()?;

    execute!(stdout(), EnterAlternateScreen)?;

    execute!(
        stdout(),
        SetBackgroundColor(Color::Blue),
        SetForegroundColor(Color::Red),
        MoveToColumn(0),
        MoveToRow(0),
        //Print("Text herererer"),
        //ResetColor
    )?;

    //execute!(stdout(), );

    deconstruct_buffer(new_buf);

    /*for xx in 0..cols {
        for yy in 0..rows {
            let (x, y) = position()?;
            if x < 10 || y < 10 {
                execute!(stdout(), Print("@"))?;
            } else {
                execute!(stdout(), Print(" "))?;
            }

        }
    }*/

    let mut x = 0;

    match read()? {
        _ => x += 1,
    }

    disable_raw_mode()?;

    execute!(stdout(), ResetColor, LeaveAlternateScreen)
}
