use rand::prelude::*;

pub struct MapGenerator {
    last_generation: Vec<Vec<u8>>,
    pub current_generation: Vec<Vec<u8>>,
    generation: u8,
    dimensions: (usize, usize),
}

impl MapGenerator {
    pub fn new(dimensions: (u16, u16)) -> MapGenerator {
        MapGenerator {
            last_generation: vec![vec![0; dimensions.1 as usize]; dimensions.0 as usize],
            current_generation: vec![vec![0; dimensions.1 as usize]; dimensions.0 as usize],
            generation: 0,
            dimensions: (dimensions.0 as usize, dimensions.1 as usize),
        }
    }

    pub fn randomize(&mut self, fill_percent: f64) {
        let mut rng = thread_rng();

        for x in 0..self.dimensions.0 {
            for y in 0..self.dimensions.1 {
                let num: f64 = rng.gen();
                if 
                    num < fill_percent ||
                    x < 2 ||
                    y < 2 ||
                    x > self.dimensions.0 - 2 ||
                    y > self.dimensions.1 - 2
                    {
                    self.current_generation[x][y] = 1;
                }
            }
        }
    }

    pub fn run_generation(&mut self) {
        self.last_generation = self.current_generation.clone();

        for (x, col) in self.last_generation.iter().enumerate() {
            for (y, val) in col.iter().enumerate() {
                let border_count = match self.get_border_count((x, y)) {
                    Some(x) => x,
                    None => continue,
                };

                if border_count < 4 {
                    self.current_generation[x][y] = 0;
                } else if border_count > 4 {
                    self.current_generation[x][y] = 1;
                } else {
                    self.current_generation[x][y] = self.last_generation[x][y];
                }

            }
        }
    }

    fn get_border_count(&self, location: (usize, usize)) -> Option<u8> {
        let (x, y) = (location.0, location.1);
        if x < 2 ||
            y < 2 ||
            x > self.dimensions.0 - 2 ||
            y > self.dimensions.1 - 2 {
            return None; // is a border must stay
        }

        let mut count = 0;

        for x in 0..=2 {
            for y in 0..=2 {
                if x == 1 && y == 1 {
                    continue; // target location dont count
                }

                println!("{}, {}", location.0 , location.1);
                count += self.last_generation[x + location.0 - 1][y + location.1 - 1];
            }
        }

        return Some(count);
    }
}

