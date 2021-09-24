use std::collections::VecDeque;

use crate::renderer::Buffer;
use crate::renderer::Dimemsion;

/*
IDEAS - 
 * redo to just do once over
*/

pub struct Influence {
    pub position: Dimemsion,
    pub matrix: Buffer,
    pub value: u32,
}

pub struct DijkstraMap {
    pub current_generation: Vec<Vec<u32>>,
    dimensions: (usize, usize),
    pub influences: Vec<Influence>,
    
}

impl DijkstraMap {
    pub fn new(dimensions: (u16, u16), influences: Vec<Influence>) -> DijkstraMap {
        DijkstraMap {
            current_generation: vec![vec![u32::MAX - 1; dimensions.1 as usize]; dimensions.0 as usize],
            dimensions: (dimensions.0 as usize, dimensions.1 as usize),
            influences,
        }
    }

    pub fn make_render(&self) -> Buffer {
        let mut result: Buffer = vec![vec![u8::MAX; self.dimensions.1]; self.dimensions.0];

        for (x, col) in self.current_generation.iter().enumerate() {
            for (y, val) in col.iter().enumerate() {
                if *val > 25 {
                    result[x][y] = 5 as u8;
                } else {
                    result[x][y] = (*val as u8) + 8;
                }
            }
        }

        return result;
    }

    pub fn apply_influences(&mut self) {
        for influence in &self.influences {
            let (column, row): (usize, usize) = (influence.position.0.into(), influence.position.1.into());

            for (x, col) in influence.matrix.iter().enumerate() {
                for (y, value) in col.iter().enumerate() {
                    if *value == 1 {
                        self.current_generation[column + x][row + y] = influence.value;
                    }
                }
            }
        }
    }

    pub fn new_implementation(&mut self) {
        self.current_generation = vec![vec![u32::MAX - 1; self.dimensions.1]; self.dimensions.0];

        self.apply_influences();

        let mut queue: VecDeque<Dimemsion> = VecDeque::new();
        let mut done: Vec<Vec<bool>>; //vec![vec![false; self.dimensions.1 as usize]; self.dimensions.0 as usize];

        let mut zero_positions: Vec<Dimemsion> = Vec::new();


        for influence in &self.influences {
            if influence.value > 100 { continue; }

            let start = influence.position;
            
            for (x, col) in influence.matrix.iter().enumerate() {
                for (y, val) in col.iter().enumerate() {
                    if *val == 1 {
                        let position = (start.0 + x as u16, start.1 + y as u16);
                        zero_positions.push(position);
                    }
                }
            }
        }   

        for position in zero_positions.clone() {
            done = vec![vec![false; self.dimensions.1 as usize]; self.dimensions.0 as usize];

            queue.push_back(position);

            for position_1 in &zero_positions {
                done[position_1.0 as usize][position_1.1 as usize] = true;
            }

            while queue.len() > 0 {
                let target = match queue.pop_front() {
                    Some(x) => x,
                    None => break,
                };
                
                let target_value = self.current_generation[target.0 as usize][target.1 as usize];
    
                for neighbour in self.get_neighbours(target).0 {
                    let position = (neighbour.0.0 as usize, neighbour.0.1 as usize);
    
                    if self.current_generation[position.0][position.1] > u32::MAX - 1 || done[position.0][position.1] {
                        continue;
                    } else {
                        queue.push_back(neighbour.0);
                        done[position.0][position.1] = true;
                        if self.current_generation[position.0][position.1] < 1 + target_value {
                            continue;
                        }
                        self.current_generation[position.0][position.1] = 1 + target_value; 
                    }
                }
            }
        }
    }

    fn get_neighbours(&self, target: Dimemsion) -> (Vec<(Dimemsion, u32)>, u32) {
        let mut neighbours = Vec::new();
        let four_dir = vec![(0, 1), (1, 0), (1, 2), (2, 1)];

        let mut lowest: u32 = u32::MAX;

        for dir in &four_dir {
            let location = (target.0 + dir.0 - 1, target.1 + dir.1 - 1);
            let value = self.current_generation[location.0 as usize][location.1 as usize];
            if value < lowest {
                lowest = value;
            }
            if value == u32::MAX {
                continue;
            }
            neighbours.push((location, value));
        }

        return (neighbours, lowest);
    }
}
