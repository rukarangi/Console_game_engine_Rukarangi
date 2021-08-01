use std::default::Default;
use crate::renderer::Buffer;

#[derive(Clone, Debug)]
pub enum ComponentList {
    Render(RenderComponent),
    Collision(CollisionComponent),
    Energy(EnergyComponent),
    Movement(MovementComponent),
}

#[derive(Clone, Debug)]
pub struct EnergyComponent {
    pub energy: u8,
}

impl EnergyComponent {
    pub fn new(energy: u8) -> EnergyComponent {
        EnergyComponent { 
            energy,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CollisionComponent {
    pub position: (u16, u16),
    //pub bottom_right: (u16, u16),
    pub matrix: Vec<Vec<u8>>,
    pub layer: u8
}

impl CollisionComponent {
    pub fn new(position: (u16, u16), matrix: Vec<Vec<u8>>, layer: u8) -> CollisionComponent {
        CollisionComponent { 
            position,
            matrix,
            layer,
        }
    }

}

#[derive(Clone, Debug)]
pub struct RenderComponent {
    pub character: u8,
    pub backgroud: u8,
    pub position_tl: (u16, u16),
    pub matrix: Buffer,
    pub visible: bool,
    pub layer: u16,
}

impl RenderComponent {
    pub fn new(character: u8, backgroud: u8, position_tl: (u16, u16), matrix: Buffer, layer: u16) -> RenderComponent {
        RenderComponent {
            character,
            backgroud,
            position_tl,
            matrix,
            visible: true,
            layer,
        }
    }

    pub fn make_render(&mut self) -> Buffer {
        let mut result: Buffer = self.matrix.clone();
        
        for (x, col) in self.matrix.iter().enumerate() {
            for (y, val) in col.iter().enumerate() {
                match val {
                    0 => result[x][y] = self.backgroud,
                    1 => result[x][y] = self.backgroud + self.character,
                    _ => result[x][y] = 0,
                }
            }
        }

        return result;
    }
}

#[derive(Clone, Debug)]
pub struct MovementComponent {
    pub desired_position: (u16, u16),
}

#[derive(Clone, Debug)]
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