use crate::components::Direction;
use crate::renderer::Buffer;

#[derive(Clone, Debug)]
pub enum AIType {
    SimpleDown,
    SimpleLeft,
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

