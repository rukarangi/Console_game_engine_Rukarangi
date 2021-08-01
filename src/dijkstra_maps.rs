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
            current_generation: vec![vec![u32::MAX; dimensions.1 as usize]; dimensions.0 as usize],
            last_generation: vec![vec![u32::MAX; dimensions.1 as usize]; dimensions.0 as usize],
            generation: 0,
            dimensions: (dimensions.0 as usize, dimensions.1 as usize),
            influences,
        }
    }

    pub fn make_render(&self) -> Buffer {
        let mut result: Buffer = vec![vec![u8::MAX; self.dimensions.1]; self.dimensions.0];

        for (x, col) in self.last_generation.iter().enumerate() {
            for (y, val) in col.iter().enumerate() {
                if *val > 33 {
                    result[x][y] = 5 as u8;
                } else {
                    result[x][y] = (*val as u8);
                }
            }
        }

        return result;
    }

    fn apply_influences(&mut self) {
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
        let mut queue = Vec::new(); // flood fill implementation WIP
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
}