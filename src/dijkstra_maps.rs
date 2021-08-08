use core::fmt::Error;
use std::collections::VecDeque;

use crate::renderer::Buffer;
use crate::renderer::Dimemsion;

pub struct Influence {
    pub position: Dimemsion,
    pub matrix: Buffer,
    pub value: u32,
}

pub struct DijkstraMap {
    pub current_generation: Vec<Vec<u32>>,
    dimensions: (usize, usize),
    pub influences: Vec<Influence>,
    permanant_desires: Vec<(Dimemsion, u32)>,
    
}

impl DijkstraMap {
    pub fn new(dimensions: (u16, u16), influences: Vec<Influence>) -> DijkstraMap {
        DijkstraMap {
            current_generation: vec![vec![u32::MAX - 1; dimensions.1 as usize]; dimensions.0 as usize],
            dimensions: (dimensions.0 as usize, dimensions.1 as usize),
            influences,
            permanant_desires: Vec::new(),
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

            if influence.value < 100 { 
                let pos_and_value:(Dimemsion, u32) = (influence.position, influence.value);
                self.permanant_desires.push(pos_and_value);
            }
        }
    }

    pub fn new_implementation(&mut self) {
        self.current_generation = vec![vec![u32::MAX - 1; self.dimensions.1]; self.dimensions.0];

        self.apply_influences();

        let mut queue: VecDeque<Dimemsion> = VecDeque::new();
        let mut covered: Vec<Vec<u32>> = vec![vec![u32::MAX; self.dimensions.1 as usize]; self.dimensions.0 as usize];

        let mut done: Vec<Vec<bool>> = vec![vec![false; self.dimensions.1 as usize]; self.dimensions.0 as usize];

        for influence in &self.influences {
            if influence.value > 100 { continue; }
            
            let start = influence.position;

            queue.push_back(start);
            done[start.0 as usize][start.1 as usize] = true;
        }

        while queue.len() > 0 {
            let target = match queue.pop_front() {
                Some(x) => x,
                None => break,
            };

            //println!("Target: {:?}", target);

            let target_value = self.current_generation[target.0 as usize][target.1 as usize];

            let mut lowest: u32 = self.get_neighbours(target).1;

            for neighbour in self.get_neighbours(target).0 {
                let position = (neighbour.0.0 as usize, neighbour.0.1 as usize);

                if self.current_generation[position.0][position.1] > u32::MAX - 1 || done[position.0][position.1] {
                    continue;
                } else {
                    queue.push_back(neighbour.0);
                    done[position.0][position.1] = true;
                    self.current_generation[position.0][position.1] = 1 + target_value; 

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

        //println!("{:?}", neighbours);

        return (neighbours, lowest);
    }

    pub fn generate_fill(&mut self) {
        self.current_generation = vec![vec![u32::MAX - 2; self.dimensions.1 as usize]; self.dimensions.0 as usize];
        self.apply_influences();

        let mut queue: VecDeque<Dimemsion> = VecDeque::new();
        let four_dir = vec![(0, 1), (1, 0), (1, 2), (2, 1)];

        for influence in &self.influences {
            if influence.value > 100 { continue; }
            
            let start = influence.position;

            for dir in &four_dir {
                queue.push_back((start.0 + dir.0 - 1, start.1 + dir.1 - 1));
            }
        }

        let mut covered: Vec<Dimemsion> = Vec::new();

        while queue.len() > 0 {

            let target = match queue.pop_front() {
                Some(x) => x,
                None => break,
            };
            if covered.iter().any(|&i| i == target) {
                continue;
            }

            covered.push(target);
            let value = self.current_generation[target.0 as usize][target.1 as usize];
            
            if value == u32::MAX {
                continue;
            }

            for desire in &self.permanant_desires {
                self.current_generation[desire.0.0 as usize][desire.0.1 as usize] = desire.1;
            }

            let desired_val = self.get_desired_value(target);
            self.current_generation[target.0 as usize][target.1 as usize] = desired_val;

            for dir in &four_dir {
                let potential_add = (target.0 + dir.0 - 1, target.1 + dir.1 - 1);
                queue.push_back(potential_add);
            }
        }

    }

    fn get_desired_value(&self, target: Dimemsion) -> u32 {
        let four_dir = vec![(0, 1), (1, 0), (1, 2), (2, 1)];

        let mut lowest = u32::MAX;

        for dir in &four_dir {
            let dir_pos = (target.0 + dir.0 - 1, target.1 + dir.1 - 1);

            let dir_value = self.current_generation[dir_pos.0 as usize][dir_pos.1 as usize];

            if dir_value < lowest {
                lowest = dir_value;
            }
        }

        //println!("{:?}", lowest);

        if lowest == u32::MAX {
            return u32::MAX-2;
        } else {
            return lowest + 1;
        }
    }

}

/*use core::fmt::Error;
use std::collections::VecDeque;

use crate::renderer::Buffer;
use crate::renderer::Dimemsion;

pub struct Influence {
    pub position: Dimemsion,
    pub matrix: Buffer,
    pub value: u32,
}

pub struct DijkstraMap {
    pub current_generation: Vec<Vec<u32>>,
    last_generation: Vec<Vec<u32>>,
    generation: u8,
    dimensions: (usize, usize),
    pub influences: Vec<Influence>,
}

impl DijkstraMap {
    pub fn new(dimensions: (u16, u16), influences: Vec<Influence>) -> DijkstraMap {
        DijkstraMap {
            current_generation: vec![vec![u32::MAX - 1; dimensions.1 as usize]; dimensions.0 as usize],
            last_generation: vec![vec![u32::MAX - 1; dimensions.1 as usize]; dimensions.0 as usize],
            generation: 0,
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
        // need way to add borders and map borders -- hmmm.

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

    pub fn generate_fill(&mut self) {
        // reset buffers
        self.current_generation = vec![vec![u32::MAX - 2; self.dimensions.1 as usize]; self.dimensions.0 as usize];
        self.apply_influences();
        self.current_generation[10][10] = 0;
        //self.last_generation = self.current_generation.clone();

        let mut queue: VecDeque<Dimemsion> = VecDeque::new();
        let four_dir = vec![(0, 1), (1, 0), (1, 2), (2, 1)];

        queue.push_back((10,9));
        queue.push_back((9,10));
        queue.push_back((11,10));
        queue.push_back((10,11));

        for influence in &self.influences {
            if influence.value > 100 { continue; }
            
            let start = influence.position;

            for dir in &four_dir {
                queue.push_back((start.0 + dir.0 - 1, start.1 + dir.1 - 1));
            }
        }

        let mut covered: Vec<Dimemsion> = Vec::new();

        let mut count = 0;
        while queue.len() > 0 {
            count += 1;
            if count >= 10 {
                break;
            }

            let target = match queue.pop_front() {
                Some(x) => x,
                None => break,
            };

            // current val
            let value = self.current_generation[target.0 as usize][target.1 as usize];
            if value == u32::MAX {
                // is a border
                continue;
            }

            let desired_val = self.get_desired_value(target);
            self.current_generation[target.0 as usize][target.1 as usize] = desired_val;

            covered.push(target);

            for dir in &four_dir {
                let potential_add = (target.0 + dir.0 - 1, target.1 + dir.1 - 1);
                if covered.iter().any(|&i| i == potential_add) {
                    continue;
                } else {
                    queue.push_back(potential_add);
                }
            }
        }

    }

    fn get_desired_value(&self, target: Dimemsion) -> u32 {
        let four_dir = vec![(0, 1), (1, 0), (1, 2), (2, 1)];

        let mut lowest = u32::MAX;

        for dir in &four_dir {
            let dir_value = self.current_generation[dir.0 as usize][dir.1 as usize];

            if dir_value < lowest {
                lowest = dir_value;
            }
        }

        if lowest == u32::MAX {
            return 22; //self.current_generation[target.0 as usize][target.1 as usize];
        } else {
            return 44;
        }
    }

    pub fn generate_fill_1(&mut self) {
        let mut queue: Vec<Dimemsion> = Vec::new(); // flood fill implementation WIP

        self.current_generation = vec![vec![u32::MAX - 2; self.dimensions.1 as usize]; self.dimensions.0 as usize];

        self.apply_influences();
        self.last_generation = self.current_generation.clone();

        let four_dir = vec![(0, 1), (1, 0), (1, 2), (2, 1)];

        for influence in &self.influences {
            if influence.value > 100 {
                continue;
            }

            let start = influence.position;
            let four_dir = vec![(0, 1), (1, 0), (1, 2), (2, 1)];

            for dir in &four_dir {
                queue.push((start.0 + dir.0 - 1, start.1 + dir.1 - 1));
            }
            //{
            /*let mut count = 0;
            while queue.len() > 0 //{
                count += 1;

                if count > 200 {
                    println!("{:?}", queue);
                    break;
                }

                let target = match queue.pop() {
                    Some(x) => x,
                    None => continue,
                }; // should never reach here without a entry
                let value = self.current_generation[target.0 as usize][target.1 as usize];
                if value == u32::MAX {
                    continue; // is a border must stay
                } 

                for dir in &four_dir {
                    let value_in_dir = self.current_generation[(target.0 + dir.0 - 1) as usize][(target.1 + dir.1 - 1) as usize];

                    if value_in_dir < value {
                        self.current_generation[target.0 as usize][target.1 as usize] = value_in_dir + 1;
                    } else {
                        queue.push((target.0 + dir.0 - 1, target.1 + dir.1 - 1));
                    }
                }
            }*/
            //}
        }

        let mut done: Vec<Dimemsion> = Vec::new();

        let mut count = 0;
        while queue.len() > 0 {
            count += 1;

            if count > 2000000 {
                println!("{:?}", queue);
                break;
            }

            let target = match queue.pop() {
                Some(x) => x,
                None => continue,
            }; // should never reach here without a entry
            done.push(target);
            let value = self.current_generation[target.0 as usize][target.1 as usize];
            if value == u32::MAX {
                continue; // is a border must stay
            } 

            for dir in &four_dir {
                let value_in_dir = self.current_generation[(target.0 + dir.0 - 1) as usize][(target.1 + dir.1 - 1) as usize];

                if value_in_dir < value {
                    self.current_generation[target.0 as usize][target.1 as usize] = value_in_dir + 1;
                } else /*if done.iter().any(|&i| i == (target.0 + dir.0 - 1, target.1 + dir.1 - 1))*/ {
                    queue.push((target.0 + dir.0 - 1, target.1 + dir.1 - 1));
                }
            }
        }
    }

    pub fn generate(&mut self) {
        for _ in 0..20 {
            self.run_generation();

            if self.current_generation == self.last_generation {
                break;
            }
        }
    }

    fn run_generation(&mut self) {
        self.apply_influences();
        self.last_generation = self.current_generation.clone();

        // apply influences
        

        for (x, col) in self.last_generation.iter().enumerate() {
            for (y, val) in col.iter().enumerate() {
                let lowest = self.get_lowest_neighbour((x, y));
                self.current_generation[x][y] = lowest;
            }
        }
    }

    fn get_lowest_neighbour(&self, location: (usize, usize)) -> u32 {
        /// returns lowest neighbour plus 1 or u32::MAX if border 
        
        let (x, y) = (location.0, location.1);
        if x < 2 ||
            y < 2 ||
            x > self.dimensions.0 - 2 ||
            y > self.dimensions.1 - 2 {
            return u32::MAX; // is a border must stay
        }

        let own = self.last_generation[location.0][location.1] as u32;
        let mut lowest = u32::MAX;

        for x in 0..=2 {
            for y in 0..=2 {
                if x == 1 && y == 1 {
                    continue; // target location dont count
                }

                let target = self.last_generation[x + location.0 - 1][y + location.1 - 1];
                
                if (target as u32) < lowest {
                    lowest = target as u32;
                }
            }
        }

        if lowest < own {
            return lowest + 1;
        } else {
            return own; // does not have higher neighbour
        }
    }
}*/