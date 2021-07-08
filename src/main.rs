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

mod state;

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


    let mut state = match state::State::new((80, 40)) { // columns, rows
        Ok(state) => state,
        Err(err) => panic!("{}", err),
    };

    let mut game = Game::new(state);

    game.move_player(Direction::Left);
    game.draw_borders();

    let field: Vec<Vec<u8>> = vec![
        vec![7; 20]; 20
    ];

    game.state.insert_matrix_at_index((35, 15), field);

    loop {
        if !game.running {
            break;
        }

        game.handle_event()?;

        game.state.draw_buffer()?;

    }

    disable_raw_mode()?;

    execute!(stdout(), ResetColor, LeaveAlternateScreen, Show)
}

fn is_event_availble() -> Result<bool> {
    poll(Duration::from_secs(0))
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Game {
    state: state::State,
    running: bool,
    borders: Vec<((u16, u16), (u16, u16), u8)>,
    gravity: u16,
    time: u32,
    player: (u16, u16),
}

impl Game {
    fn new(state_init: state::State) -> Game {
        let mut state = state_init;
        
        let running = true;
        let borders = vec![
            ((1, 1), (state.dimensions.0, 1), 3), // (80, 1)
            ((1, 1), (1, state.dimensions.1), 3), // (1, 40)
            // (80, 1) (80, 40) = (1, 38)
            ((state.dimensions.0, 1), (state.dimensions.0, state.dimensions.1), 3),
            // (1, 40) (80, 40) = (78, 1)
            ((1, state.dimensions.1), (state.dimensions.0, state.dimensions.1), 3),
            ((20, 20), (30, 30), 3),
        ];

        let gravity = 1;
        let time = 0;

        let player = (5, 5);

        return Game {
            state,
            running,
            borders,
            gravity,
            time,
            player,
        };
    }

    fn draw_borders(&mut self) {
        for border in &self.borders {
            let mut length: usize;
            if border.1.0 == border.0.0 {
                length = 1;
            } else {
                length = (border.1.0 - border.0.0 + 1 ).into();
            }
            let mut height: usize;
            if border.1.1 == border.0.1 {
                height = 1;
            } else {
                height = (border.1.1 - border.0.1 + 1 ).into();
            }

            let mut start = border.0;

            let border_real: Vec<Vec<u8>> = vec![vec![border.2; height]; length];

            self.state.insert_matrix_at_index(start, border_real);
        }
    }

    fn handle_event(&mut self) -> Result<()>{
        match is_event_availble() {
            Ok(true) => {
                match read()? {
                    Event::Key(key_event) => {
                        match key_event.code {
                            KeyCode::Char(c) => {
                                match c {
                                    'q' => self.running = false,
                                    _ => return Ok(()),
                                }
                            },
                            KeyCode::Right => {
                                self.move_player(Direction::Right);
                            },
                            KeyCode::Left => {
                                self.move_player(Direction::Left);
                            },
                            KeyCode::Up => {
                                self.move_player(Direction::Up);
                            },
                            KeyCode::Down => {
                                self.move_player(Direction::Down);
                            },
                            _ => return Ok(()),
                        }
                    },
                    _ => return Ok(()),
                }
            },
            _ => return Ok(()),
        }

        Ok(())
    }
    
    fn move_player(&mut self, direction: Direction) {
        let proposed_pos: (u16, u16) = match direction {
            Direction::Up => {
                (self.player.0, self.player.1 - 1)
            },
            Direction::Down => {
                (self.player.0, self.player.1 + 1)
            },
            Direction::Left => {
                (self.player.0 - 1, self.player.1)
            },
            Direction::Right => {
                (self.player.0 + 1, self.player.1)
            },
        };

        for border in &self.borders {
            if (proposed_pos.0 >= border.0.0 && // is right of top left
                proposed_pos.0 <= border.1.0 && // is left of bottom right
                proposed_pos.1 >= border.0.1 && // is down of top left
                proposed_pos.1 <= border.1.1) { // is up of bottom right
                return; // is colliding dont move
            }
        }

        let mut last_color: u8;

        if self.state.alternate_buffer[self.player.0 as usize][self.player.1 as usize] < 10 {
            last_color = 0;
        } else {
            last_color = self.state.alternate_buffer[self.player.0 as usize][self.player.1 as usize] - 10; // fix bug

        }

        let proper_color = self.state.alternate_buffer[proposed_pos.0 as usize][proposed_pos.1 as usize] + 10;

        self.state.update_char_at_index(proposed_pos, proper_color);
        
        self.state.update_char_at_index(self.player, last_color); // will overwrite

        self.player = proposed_pos;
    }

}