use std::default::Default;

#[derive(Clone, Debug)]
pub struct CollisionComponent {
    pub position: (u16, u16),
    //pub bottom_right: (u16, u16),
    pub matrix: Vec<Vec<u8>>,
}

impl CollisionComponent {
    pub fn new(position: (u16, u16), matrix: Vec<Vec<u8>>) -> CollisionComponent {
        CollisionComponent { 
            position,
            matrix,
        }
    }   
}

#[derive(Clone, Debug)]
pub struct RenderComponent {
    pub character: u8,
    pub position_tl: (u16, u16),
    pub position_br: (u16, u16),
    pub visible: bool,
    pub layer: u16,
}

impl RenderComponent {
    pub fn new(character: u8, position_tl: (u16, u16), position_br: (u16, u16), layer: u16) -> RenderComponent {
        RenderComponent {
            character,
            position_tl,
            position_br,
            visible: true,
            layer,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MovementComponent {
    pub desired_position: (u16, u16),
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl MovementComponent {
    pub fn new(position: (u16, u16)) -> MovementComponent {
        MovementComponent { 
            desired_position: position, 
        }
    }

    pub fn move_desired(&mut self, direction: Direction) {
        match direction {
            Direction::Up => {
                //self.desired_position.0; 
                self.desired_position.1 -= 1;
            },
            Direction::Down => {
                //self.desired_position.0; 
                self.desired_position.1 += 1;
            },
            Direction::Left => {
                self.desired_position.0 -= 1; 
                //self.desired_position.1;
            },
            Direction::Right => {
                self.desired_position.0 += 1; 
                //self.desired_position.1;
            },
        }
    }
}