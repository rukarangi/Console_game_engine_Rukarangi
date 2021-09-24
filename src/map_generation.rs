use rand::prelude::*;
use rand::seq::SliceRandom;
use crate::components::Direction;
use std::collections::HashMap;
use std::collections::VecDeque;

// Cave-like generation using automata
pub struct CaveMapGenerator {
    last_generation: Vec<Vec<u8>>,
    pub current_generation: Vec<Vec<u8>>,
    dimensions: (usize, usize),
}

impl CaveMapGenerator {
    pub fn new(dimensions: (u16, u16)) -> CaveMapGenerator {
        CaveMapGenerator {
            last_generation: vec![vec![0; dimensions.1 as usize]; dimensions.0 as usize],
            current_generation: vec![vec![0; dimensions.1 as usize]; dimensions.0 as usize],
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

    pub fn generate(&mut self) {
        for _ in 0..10 {
            self.run_generation();

            if self.current_generation == self.last_generation {
                break;
            }
        }
    }

    pub fn make_render(&self) -> Vec<Vec<u8>> {
        let mut result = self.last_generation.clone();

        for (x, col) in self.last_generation.iter().enumerate() {
            for (y, val) in col.iter().enumerate() {
                match val {
                    0 => {
                        result[x][y] = 1;
                    },
                    1 => {
                        result[x][y] = 3;
                    },
                    _ => {
                        result[x][y] = 3;
                    },
                }
            }
        }
        
        return result;
    }

    fn run_generation(&mut self) {
        self.last_generation = self.current_generation.clone();

        for (x, col) in self.last_generation.iter().enumerate() {
            for (y, val) in col.iter().enumerate() {
                let border_count = match self.get_border_count((x, y)) {
                    Some(x) => x,
                    None => continue,
                };

                match val {
                    0 => {
                        if border_count >= 5 {
                            self.current_generation[x][y] = 1;
                        } else {
                            self.current_generation[x][y] = 0;
                        }
                    },
                    1 => {
                        if border_count >= 4 {
                            self.current_generation[x][y] = 1;
                        } else {
                            self.current_generation[x][y] = 0;
                        }
                    },
                    _ => continue,
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

                //println!("{}, {}", location.0 , location.1);
                count += self.last_generation[x + location.0 - 1][y + location.1 - 1];
            }
        }

        return Some(count);
    }
}

// WFC generation

// Data Structs
#[derive(Debug)]
pub struct AdjanencyRule(HashMap<(usize, usize), Vec<usize>>);
pub type AdjanencyRules = Vec<AdjanencyRule>;
pub type Tile = [[u8; 3]; 3];

#[derive(Debug)]
pub struct WaveFormCollapser {
    adjanency_rules: Option<AdjanencyRules>,
    in_progress_map: Vec<Vec<Vec<usize>>>,
    queue: VecDeque<(usize, usize)>,
    output: Vec<Vec<u8>>,
    index_character_map: HashMap<usize, u8>,
    tiles: usize,
    dimensions: (usize, usize),
}

impl AdjanencyRule {
    fn new(entries: Vec<((usize, usize), Vec<usize>)>) -> AdjanencyRule {
        let mut new_rule: HashMap<(usize, usize), Vec<usize>> = HashMap::new();

        for entry in entries {
            new_rule.insert(entry.0, entry.1);
        }

        return AdjanencyRule(new_rule);
    }

    fn allow(&mut self, dir: (usize, usize), tile: usize) {
        let mut x = match self.get(&dir).clone() {
            Some(x) => x.clone(),
            None => Vec::new(),
        };
        x.push(tile);

        self.0.remove(&dir);
        self.0.insert(dir, x);
    }
    fn get(&self, key: &(usize, usize)) -> Option<&Vec<usize>> {
        match self.0.get(key) {
            Some(val) => return Some(val),
            None => return None,
        }
    }

    fn view(&self) {
        let left = format!("{:?}", self.get(&(0, 1)).unwrap());
        let top = format!("{:?}", self.get(&(1, 0)).unwrap());
        let bottom = format!("{:?}", self.get(&(1, 2)).unwrap());
        let right = format!("{:?}", self.get(&(2, 1)).unwrap());

        println!("Left: {:<10} Top: {:<10} Bottom: {:<10} Right: {:<10}\n",
            left, top, bottom, right);
    }
}

impl WaveFormCollapser {
    pub fn new() ->  WaveFormCollapser {
        WaveFormCollapser {
            adjanency_rules: None,
            in_progress_map: Vec::new(),
            queue: VecDeque::new(),
            output: Vec::new(),
            index_character_map: HashMap::new(),
            tiles: 0,
            dimensions: (0,0),
        }
    }

    // utility inside impl
    fn allowed_neighbours(&self, target:(usize, usize), dir: &(usize, usize)) -> Vec<usize> {
        let mut allowed_neighbours: Vec<usize> = Vec::new();

        for potential_value in &self.in_progress_map[target.0][target.1] {
            match &self.adjanency_rules {
                Some(rules) => {
                    let rule = &rules[potential_value.clone()];

                    for val in rule.get(dir).unwrap() {
                        allowed_neighbours.push(*val);
                    }
                },
                None => panic!("Ahh no rules yet"),
            }
        }

        return allowed_neighbours;
    }

    fn is_complete(&self) -> bool {
        let mut complete = true;

        for x in &self.in_progress_map {
            for y in x {
                if y.len() > 1 {
                    complete = false;
                }
            }
        }

        return complete;
    }

    // actual process
    pub fn pre_wfc_3x3(&mut self, input: Vec<Vec<u8>>) {
        // assume that it is square
        let eight_dir: Vec<(usize, usize)> = vec![
            (0, 0), (1, 0), (2, 0), (0, 1),
            (2, 1), (0, 2), (1, 2), (2, 2),
        ];
        let four_dir: Vec<(usize, usize)> = vec![(0, 1), (1, 0), (1, 2), (2, 1)];

        let num_tiles_width: usize = input.len() - 2;
        let mut tiles: Vec<Tile> = Vec::new();

        let mut adjanency_rules: AdjanencyRules = Vec::new();

        for x in 0..num_tiles_width {
            for y in 0..num_tiles_width {
                let mut new_tile: Tile = [[0; 3]; 3];

                let centre = (x + 1, y + 1);
                new_tile[1][1] = input[centre.0][centre.1];

                for dir in &eight_dir {
                    let map_val = input[dir. 0 + centre.0 - 1][dir. 1 + centre.1 - 1];
                    new_tile[dir.0][dir.1] = map_val;
                }

                //tiles.push(new_tile);
                let tile_versions = get_reflections(&new_tile);

                for tile in tile_versions {
                    if !tiles.contains(&tile) {
                        tiles.push(tile);
                    }
                }
                // WIP HAVE TO ADD RELFECTIONS AND REMOVE IDENTICALS
            }
        }

        //println!("\n\n--------\nTILES:\n--------");
        for (idx, tile) in tiles.clone().iter().enumerate() {
            //println!("\nTile: {}", idx);
            //print_buf_8(tile);
        }

        for (i, a) in tiles.iter().enumerate() {
            let mut rule: AdjanencyRule = AdjanencyRule::new(Vec::new());

            for (ib, b) in tiles.clone().iter().enumerate() {
                for (dir_i, dir) in vec![Direction::Left, Direction::Up, Direction::Down, Direction::Right].iter().enumerate() {
                    if compatible(a, b, dir) {
                        rule.allow(four_dir[dir_i], ib);
                    }
                }
            }

            //println!("{}: ", i);
            //rule.view();
            //println!("{:?}", rule);
            
            self.index_character_map.insert(i, tiles[i][1][1]);
            adjanency_rules.push(rule);
        }

        self.adjanency_rules = Some(adjanency_rules);
        self.tiles = tiles.len();
    }

    pub fn wfc_core(&mut self, output_size: (usize, usize)) {
        self.dimensions = output_size;
        let mut rng = thread_rng();
        // Left, Up, Down, Right
        let four_dir: Vec<(usize, usize)> = vec![(0, 1), (1, 0), (1, 2), (2, 1)];

        self.in_progress_map = vec![vec![(0..self.tiles).collect(); output_size.1]; output_size.0];

        let mut start_coord: (usize, usize) = (rng.gen_range(0..output_size.0) as usize, rng.gen_range(0..output_size.1) as usize);
        let mut first_tile: usize = rng.gen_range(0..self.tiles);

        // for testing
        //start_coord = (8,5);
        //first_tile = 8;

        //println!("{:?}: {:?}", start_coord, first_tile);

        self.in_progress_map[start_coord.0][start_coord.1] = vec![first_tile];

        //println!("\nFirst Tile:\n");
        //print_map(&self.in_progress_map);

        self.queue.push_back(start_coord);

        let width = output_size.0 - 1;
        let height = output_size.1 - 1;
        let mut count = 0;
        while !self.is_complete() {

            count += 1;

            //println!("\n---------\nITERATION #{}\n---------\n\nBefore:", count);
            //print_map(&self.in_progress_map);
            //println!("");

            if self.queue.len() > 0 {

                //println!("Started Tile Propagation");

                let current_position: (usize, usize) = self.queue.pop_front().unwrap();

                //println!("at: {:?}", current_position);

                let mut open_neighbours: Vec<bool> = vec![true, true, true, true];
                    
                match current_position.0 {
                    0 => open_neighbours[0] = false,
                    _ if current_position.0 == width => open_neighbours[3] = false,
                    _ => (),
                }

                match current_position.1 {
                    0 => open_neighbours[1] = false,
                    _ if current_position.1 == height => open_neighbours[2] = false,
                    _ => (),
                }

                //println!("\nChecking Direction Now\n");
                for (idx, dir) in four_dir.iter().enumerate() {
                    let allowed_neighbours: Vec<usize> = self.allowed_neighbours(current_position, dir);

                    if open_neighbours[idx] {
                        // can find neighbour in this direciton

                        let mut needs_filtering = false;

                        for i in &self.in_progress_map[current_position.0 + dir. 0 - 1][current_position.1 + dir. 1 - 1] {
                            if !allowed_neighbours.contains(&i) {
                                // condition met if neighbour has illegal potential results
                               
                                needs_filtering = true;

                                break;
                            }
                        }
                        
                        if needs_filtering {   
                            self.in_progress_map
                                [current_position.0 + dir. 0 - 1]
                                [current_position.1 + dir. 1 - 1]
                                .retain(|x| allowed_neighbours.contains(x));

                            let bad_neighbour = (current_position.0 + dir. 0 - 1, current_position.1 + dir. 1 - 1);
                            self.queue.push_back(bad_neighbour);
                        }
                    }
                }
                
                //println!("\nAfter {:?}\n\nQueue: {:?}\n", current_position, self.queue);
                //print_map(&self.in_progress_map);

            } else {
                //println!("\n---------\nEMPTY QUEUE\nRE-PICK\n---------");

                let new_pick = match find_lowest_non_one_pos(&self.in_progress_map) { // slow
                    Some(pos) => pos,
                    None => break,
                };

                self.in_progress_map[new_pick.0][new_pick.1] = vec![*self.in_progress_map[new_pick.0][new_pick.1].choose(&mut rng).unwrap()];
                self.queue.push_back(new_pick);

                //println!("---------\nNEW START: {:?}\n---------\n", new_pick);                
            }
        }

        //print_tiles_map(&self.in_progress_map, &self.index_character_map);
    }

    pub fn post_wfc(&mut self) {
        self.output = vec![vec![0; self.dimensions.1]; self.dimensions.0];

        for x in 0..self.dimensions.0 {
            for y in 0..self.dimensions.1 {
                self.output[x][y] = *self.index_character_map.get(&self.in_progress_map[x][y][0]).unwrap();
            }
        }
    }
}

// General Ultility
pub fn find_lowest_non_one_pos(map: &Vec<Vec<Vec<usize>>>) -> Option<(usize, usize)> {
    let mut rng = thread_rng();
    
    let mut lowest_value: usize = 10000;
    let mut lowest_positions: Vec<(usize, usize)> = Vec::new();
    
    // really annoying to loop twice,
    // need learn better algo

    for (x, col) in map.iter().enumerate() {
        for (y, val) in col.iter().enumerate() {
            if val.len() < lowest_value && val.len() > 1 {
                lowest_value = val.len();
            }
        }
    }

    for (x, col) in map.iter().enumerate() {
        for (y, val) in col.iter().enumerate() {
            if val.len() == lowest_value {
                lowest_positions.push((x, y));
            }
        }
    }

    let lowest_pos = lowest_positions.choose(&mut rng).unwrap() ;

    if lowest_value == 10000 {
        return None;
    } else {
        return Some(*lowest_pos);
    }
}

fn print_buf_8(buffer: &Tile) {
    for i in buffer {
        let mut line: String = String::new();
        for x in i {
            line.push_str(&format!("{}", x));
            line.push(' ');
        }
        println!("{}", line);
    }
}

fn print_tiles_map(buffer: &Vec<Vec<Vec<usize>>>, hash: &HashMap<usize, u8>) {
    println!("\n---------\nDONE MAP\n---------\n");
    for x in 0..buffer.len() {
        let mut new_line = String::new();
        for y in 0..buffer[0].len() {
            new_line.push_str(&format!("{:<2?}", hash.get(&buffer[x][y][0]).unwrap()));
            new_line.push(' ');
        } 
        println!("{}", new_line);
    }
}

fn print_map(buffer: &Vec<Vec<Vec<usize>>>) {
    for x in 0..buffer[0].len() {
        let mut new_line = String::new();
        for y in 0..buffer[0].len() {
            new_line.push_str(&format!("{:<2}", buffer[x][y].len()));
            new_line.push(' ');
        }
        println!("{}", new_line);
    }
}

// Tile ultility

fn compatible(a: &[[u8; 3]; 3], b: &[[u8; 3]; 3], direction: &Direction) -> bool {
    match direction {
        Direction::Up => {
            let cross_a: [[u8; 3]; 2] = [
                [a[0][0], a[1][0], a[2][0]],
                [a[1][1], a[1][1], a[2][1]],
            ];
            let cross_b: [[u8; 3]; 2] = [
                [b[0][1], b[1][1], b[2][1]],
                [b[1][2], b[1][2], b[2][2]],
            ];

            return cross_a == cross_b;
        },
        Direction::Down => {
            let cross_a: [[u8; 3]; 2] = [
                [a[0][1], a[1][1], a[2][1]],
                [a[1][2], a[1][2], a[2][2]],
            ];
            let cross_b: [[u8; 3]; 2] = [
                [b[0][0], b[1][0], b[2][0]],
                [b[1][1], b[1][1], b[2][1]],
            ];

            return cross_a == cross_b;
        },
        Direction::Left => {
            let cross_a: [[u8; 3]; 2] = [a[1], a[2]];
            let cross_b: [[u8; 3]; 2] = [b[0], b[1]];

            return cross_a == cross_b;
        },
        Direction::Right => {
            let cross_a: [[u8; 3]; 2] = [a[0], a[1]];
            let cross_b: [[u8; 3]; 2] = [b[1], b[2]];

            return cross_a == cross_b;
        },
    }
}

fn transpose(tile: &Tile) -> Tile {
    let mut tranposed = tile.clone();

    for x in 0..tile.len() {
        for y in 0..tile.len() {
            tranposed[y][x] = tile[x][y];
        }
    }

    return tranposed;
}

fn reverse_x(tile: &Tile) -> Tile {
    let mut reversed = tile.clone();
    
    for x in 0..tile.len() {
        reversed[x].reverse();
    }

    return reversed;
}

pub fn get_reflections(tile: &Tile) -> Vec<Tile> {
    let mut reflections = Vec::new();

    // no action, 0 degrees
    reflections.push(tile.clone());

    // 90 degrees, transpose, reverse 
    let mut tile_90: Tile = tile.clone();
    tile_90 = reverse_x(&transpose(tile));

    reflections.push(tile_90);

    // -90 degrees, reverse, transpose
    let mut tile_neg90: Tile = tile.clone();
    tile_neg90 = transpose(&reverse_x(tile));

    reflections.push(tile_neg90);

    // 180 degrees, transpose, reverse, transpose, reverse
    let mut tile_180: Tile = tile.clone();
    tile_180 = reverse_x(&transpose(&reverse_x(&transpose(tile))));

    reflections.push(tile_180);

    //println!("\nTile Relfections\n\n{:?}\n\nRotated 90\n\n{:?}\n\nRotated -90\n\n{:?}\n\nRotated 180\n\n{:?}", tile, tile_90, tile_neg90, tile_180);

    return reflections;
}