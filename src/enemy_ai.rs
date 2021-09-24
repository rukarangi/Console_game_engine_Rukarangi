use crate::components::Direction;
use crate::renderer::Buffer;
use rand::prelude::*;

#[derive(Clone, Debug)]
pub enum AIType {
    SimpleDown,
    SimpleLeft,
    RollDownPlayer,
}

#[derive(Clone, Debug)]
pub struct EnemyAIComponent {
    pub ai_type: AIType,
}

impl EnemyAIComponent {
    pub fn new(ai_type: AIType) -> EnemyAIComponent {
        EnemyAIComponent {
            ai_type,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Action {
    DoNothing,
    Move(Direction),
}

pub fn simple_down(position: (u16, u16), collision_buffer: &Buffer) -> (Action, u8) {
    // cant move down
    if collision_buffer[position.0 as usize - 1][position.1 as usize] == 1 {
        return (Action::DoNothing, 10);
    } else { // can move down
        return (Action::Move(Direction::Down), 10);
    }
}

pub fn roll_down_player(position: (u16, u16), collision_buffer: &Buffer, player_dijk: &Vec<Vec<u32>>) -> (Action, u8) {
    let lowest_direction = lowest_direction(position, player_dijk);

    match lowest_direction {
        Some(direction) => {
            if collision_buffer[(position.0 + direction.0.0 - 1) as usize][(position.1 + direction.0.1 - 1) as usize] == 1 {
                return (Action::DoNothing, 10);
            } else { // can move in direction
                return (Action::Move(direction.1), 10);
            }
        },
        None => return (Action::DoNothing, 10),
    }
}

fn lowest_direction(target: (u16, u16), dijk: &Vec<Vec<u32>>) -> Option<((u16, u16), Direction)> {
    let four_pos = vec![(0, 1), (1, 0), (1, 2), (2, 1)];
    let four_dir = vec![Direction::Left, Direction::Up, Direction::Down, Direction::Right];

    let mut lowest = u32::MAX;

    let four_values: Vec<(usize, u32)> = four_pos
        .iter()
        .enumerate()
        .map(|(i, x)| (i, dijk[(x.0 + target.0 - 1) as usize][(x.1 + target.1 - 1) as usize]))
        .collect();

    for (_, x) in four_values.iter() {
        if lowest > *x {
            lowest = *x;
        }
    }

    let equal_lowest: Vec<&(usize, u32)> = four_values
        .iter()
        .filter(|(_i, x)| x <= &lowest)
        .collect();

    let mut rng = thread_rng();

    match equal_lowest.len() {
        1 => return Some((four_pos[equal_lowest[0].0], four_dir[equal_lowest[0].0])),
        2 | 3 | 4 => {
            let random_idx = rng.gen_range(0..equal_lowest.len());

            return Some(
                (four_pos[equal_lowest[random_idx].0], four_dir[equal_lowest[random_idx].0])
            );
        },
        _ => return None,
    }
}
