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

//const U8TOCHAR: [StyledContent<char>; 4] = [' '.on_blue(), '.'.on_green(), '@'.on_blue(), '#'.on_blue()];

fn reset_screen(x: u16, y: u16) {
    execute!(
        stdout(),
        Clear(All),
        MoveToColumn(x),
        MoveToRow(y),
    );
}

fn display_buffer(buf: &Vec<Vec<u8>>) {
    let u8_to_char: [StyledContent<char>; 4] = [' '.on_blue(), '.'.on_green(), '@'.on_blue(), '#'.on_blue()];

    reset_screen(0, 0);

    let mut copy = buf.clone();

    copy[0][0] = 1;
    copy[0][3] = 1;
    copy[0][4] = 1;
    copy[0][5] = 1;
    
    for row in copy.iter() {
        for c in row.iter() {
            //execute!(stdout(), Print(u8_to_char[*c as usize]));
            execute!(stdout(), Print(*c));
        }
    }
}

fn buffer_to_view(buf: Vec<Vec<u8>>, top_left: (usize, usize), bottom_right: (usize, usize)) -> Vec<Vec<u8>> {
    let mut view: Vec<Vec<u8>> = Vec::new();

    let iter = buf[top_left.1..bottom_right.1].iter().map(|s| &s[top_left.0..bottom_right.0]);

    for row in iter {
        let mut new_row: Vec<u8> = Vec::new();
        for x in row {
            new_row.push(*x);
        }
        view.push(new_row);
    }

    return view;
}

fn get_initial_buffer() -> Vec<Vec<u8>> {
    let mut result = vec![vec![0; 100]; 300];

    /*for (idx1, x) in result.clone().iter().enumerate() {
        for (idx2, y) in x.iter().enumerate() {
            if y % 30 == 0 {
                result[idx1][idx2] = 1;
            }
        }
    }*/

    result[0][2] = 1;
    result[0][3] = 1;
    result[0][4] = 1;
    result[0][5] = 1;
    //result[20][20] = 2;
    //result[20][21] = 2;
    //result[21][21] = 2;
    //result[21][20] = 2;

    return result;
}

fn main() -> Result<()> {
    execute!(stdout(), SetSize(100, 300))?;

    let (cols, rows): (u16, u16) = (100, 300);//size()?;

    let mut buf: Vec<Vec<u8>> =  vec![vec![0; cols.into()]; rows.into()]; //get_initial_buffer();

    buf[0][2] = 1;
    buf[0][3] = 1;
    buf[0][4] = 1;
    buf[0][5] = 1;

    /*let mut new_buf: Vec<Vec<u8>> = Vec::new(); //vec![vec![0; rows.into()]; cols.into()];

    for (index, row) in buf.clone().iter().enumerate() {
        new_buf.push(Vec::new());
        for (idx, c) in row.iter().enumerate() {
            if index < 95 {
                new_buf[index].push(0);
            } else {
                new_buf[index].push(buf[index][idx]);
            }
        }
    }*/

    enable_raw_mode()?;

    execute!(stdout(), EnterAlternateScreen)?;

    execute!(
        stdout(),
        SetBackgroundColor(Color::Blue),
        SetForegroundColor(Color::Red),
        //MoveToColumn(0),
        //MoveToRow(0),
    )?;

    display_buffer(/*buffer_to_view(*/&buf/*, (0, 0), (100, 300))*/);

    //reset_screen(0,0);

    //execute!(stdout(), Print(buf[0][1]), Print(buf[0][0]));

    let mut x = 0;

    match read()? {
        _ => x += 1,
    }

    disable_raw_mode()?;

    execute!(stdout(), ResetColor, LeaveAlternateScreen)
}
