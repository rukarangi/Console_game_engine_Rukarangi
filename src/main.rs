use std::time::Duration;
use std::io::{stdout, Write};

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

mod generations;
use crate::generations::GenerationalIndex;
use crate::generations::GenerationalIndexAllocator;
mod renderer;
use crate::renderer::*;
mod components;
use crate::components::*;
mod enemy_ai;
use crate::enemy_ai::*;
mod map_generation;
use crate::map_generation::*;
mod dijkstra_maps;
use crate::dijkstra_maps::*;

type EntityMap<T> = generations::GenerationalIndexArray<T>;
pub type Entity = generations::GenerationalIndex;

/*
Notes: 
after adding map, for blindness add high layer of "blind"
then use collision buffer and local to remove parts,
thus making it clear and visible.

ok move working, now fields things that arent collider but visible

IDEA:

fn handle_comps(&mut self) {
    for gen_index in self.entity_allocator.get_vec() {
        let comp_x = self.x_components.get_mut(gen_index);
        ...

        match (comp_x, comp_y, ..) {
            (Some(x), Some(y)) => handle_x_y(x, y),
            (Some(x), None) => handle_x(x),
            ..
        }

    }
}

*/

fn main() -> Result<()> {
    let style_map: StyleMap = vec![ // 1 for character, 2 for wall, 0 for floor
        '^'.on(Color::Red),  // test
        ' '.on(Color::Grey), // 1
        '@'.on(Color::Grey), // 2
        '#'.on(Color::Grey), // 3
        '*'.on(Color::Grey), // 4
        ' '.on(Color::Blue), // 5
        '@'.on(Color::Blue), // 6
        '#'.on(Color::Blue), // 7
        '*'.on(Color::Blue), // 8
        ' '.on(Color::AnsiValue(255)),  // dijk stuff
        ' '.on(Color::AnsiValue(254)), // 10
        ' '.on(Color::AnsiValue(253)), // 11
        ' '.on(Color::AnsiValue(252)), // 12
        ' '.on(Color::AnsiValue(251)), // 13
        ' '.on(Color::AnsiValue(250)), // 14
        ' '.on(Color::AnsiValue(249)), // 15
        ' '.on(Color::AnsiValue(248)), // 16
        ' '.on(Color::AnsiValue(247)), // 17
        ' '.on(Color::AnsiValue(246)), // 18
        ' '.on(Color::AnsiValue(245)), // 19
        ' '.on(Color::AnsiValue(244)), // 20
        ' '.on(Color::AnsiValue(243)), // 21
        ' '.on(Color::AnsiValue(242)), // 22
        ' '.on(Color::AnsiValue(241)), // 23
        ' '.on(Color::AnsiValue(240)), // 24
        ' '.on(Color::AnsiValue(239)), // 25
        ' '.on(Color::AnsiValue(238)), // 26
        ' '.on(Color::AnsiValue(237)), // 27
        ' '.on(Color::AnsiValue(236)), // 28
        ' '.on(Color::AnsiValue(235)), // 29
        ' '.on(Color::AnsiValue(234)), // 30
        ' '.on(Color::AnsiValue(233)), // 31
        ' '.on(Color::AnsiValue(232)), // 32
        ' '.on(Color::AnsiValue(231)), // 33
    ];

    let dimensions: Dimemsion = (200, 100);
    let view_port: Dimemsion = (150, 60);

    let mut game: GameState = GameState::new(dimensions, view_port, style_map);

    let random_map = false;
    let test_collison = false;
    let test_dijk = true;

    game.init_player((5, 5));
    game.init_test_enemy((22, 18));
    game.init_test_enemy((40, 40));
    game.init_test_enemy((22, 15));

    if random_map {
        game.init_map(0.45); // 0.5 is good
    } else {
        game.init_borders();
        game.init_background();
        game.init_field();
    }

    while game.running {
        if is_event_availble()? {
            game.handle_input(read()?);
        }

        game.handle_collision();
        game.handle_movement();
        game.test_influences();
        game.handle_enemy_energy_move();
        game.handle_render();
        game.renderer.render()?;
        
    }

    if test_collison {    
        game.running = true;

        while game.running {
            if is_event_availble()? {
                game.handle_input(read()?);
            }

            game.handle_collision();

            game.renderer.insert_matrix((0, 0), &game.collision_buffer);
            game.renderer.render()?;
        }
    }

    if test_dijk{
        game.running =  true;

        game.test_influences();

        while game.running {
            if is_event_availble()? {
                game.test_influences();
                game.handle_input(read()?);
            }

            game.handle_collision();
            game.handle_movement();

            
            //game.renderer.
            game.renderer.insert_matrix((0, 0), &game.player_dijk.make_render());
            game.renderer.render()?;
        }
    }

    Renderer::reset_term()?;

    Ok(())
}

struct GameState {
    // resources
    renderer: Renderer,
    collision_buffer: Buffer,
    player_dijk: DijkstraMap,
    running: bool,
    empty_buffer: Buffer,
    map_generator: MapGenerator,

    // ECS
    entity_allocator: GenerationalIndexAllocator,
    // for now assume every entity has render comp
    render_components: EntityMap<RenderComponent>,
    movement_components: EntityMap<MovementComponent>,
    collision_components: EntityMap<CollisionComponent>,
    energy_components: EntityMap<EnergyComponent>,
    enemy_ai_components: EntityMap<EnemyAIComponent>,

    // Player
    player: Option<Entity>,
}

impl GameState {
    fn new(dimensions: Dimemsion, view_port: Dimemsion, style_map: StyleMap) -> GameState {
        // resources
        let renderer =  match renderer::Renderer::initialize(dimensions, view_port, style_map) {
            Ok(r) => r,
            Err(err) => panic!("Failed Renderer Intialisation: {}", err),
        };
        let collision_buffer = renderer.input_buffer.clone();
        let player_dijk = DijkstraMap::new(dimensions, Vec::new());
        let empty_buffer = renderer.input_buffer.clone();
        let map_generator = MapGenerator::new(dimensions);
        
        // ECS
        let entity_allocator = generations::GenerationalIndexAllocator::new(1);
        let render_components = EntityMap::<RenderComponent>::new();
        let movement_components = EntityMap::<MovementComponent>::new();
        let collision_components = EntityMap::<CollisionComponent>::new();
        let energy_components = EntityMap::<EnergyComponent>::new();
        let enemy_ai_components = EntityMap::<EnemyAIComponent>::new();
        
        GameState {
            renderer,
            collision_buffer,
            player_dijk,
            running: true,
            empty_buffer,
            map_generator,
            entity_allocator,
            render_components,
            movement_components,
            collision_components,
            energy_components,
            enemy_ai_components,
            player: None,
        }
    }

    fn test_influences(&mut self) {
        let player = match self.player {
            Some(e) => e,
            None => panic!("No player!"),
        };

        let comp = match self.render_components.get_mut(player) {
            Some(comp) => comp,
            None => panic!("No player!"),
        };
        let col_influence: Influence = Influence {
            position: (0, 0),
            matrix: self.collision_buffer.clone(),
            value: u32::MAX,
        };
        let player_influence: Influence = Influence {
            position: comp.position_tl,
            matrix: vec![vec![1]],
            value: 0,
        };
        let player_influence_2: Influence = Influence {
            position: (comp.position_tl.0 + 5, comp.position_tl.1 + 5),
            matrix: vec![
                vec![1;4],
            ],
            value: 0,
        };
        
        self.player_dijk.influences = vec![col_influence, player_influence, player_influence_2];
        //self.player_dijk.generate();
        //self.player_dijk.generate_fill();
        self.player_dijk.new_implementation();
    }

    fn test_movement(&mut self) {
        match self.player {
            Some(player_entity) => {
                match self.movement_components.get_mut(player_entity) {
                    Some(component) => {
                        component.move_desired(Direction::Right);
                    }
                    None => panic!("No movement comp on player!"),
                }
            },
            None => panic!("No player!"),
        }
    }

    fn move_entity(&mut self, entity: Entity, direction: Direction) {
        match self.movement_components.get_mut(entity) {
            Some(component) => {
                component.move_desired(direction);
            },
            None => panic!("No movement comp on entity!"),
        }
    }

    fn handle_input(&mut self, input: Event) {
        let player = match self.player {
            Some(e) => e,
            None => panic!("No player!"),
        };
        match input {
            Event::Key(key_event) => {
                match key_event.code {
                    KeyCode::Char('q') => self.running = false,
                    KeyCode::Right => {
                        self.move_entity(player, Direction::Right);
                        self.add_ten_energy();
                    },
                    KeyCode::Left => {
                        self.move_entity(player, Direction::Left);
                        self.add_ten_energy();
                    },
                    KeyCode::Up => {
                        self.move_entity(player, Direction::Up);
                        self.add_ten_energy();
                    },
                    KeyCode::Down => {
                        self.move_entity(player, Direction::Down);
                        self.add_ten_energy();
                    },
                    _ => return,
                }
            },
            _ => return,
        }
    }

    fn add_ten_energy(&mut self) {
        for gen_index in self.entity_allocator.get_vec() {
            let comp = match self.energy_components.get_mut(gen_index) {
                Some(comp) => comp,
                None => continue,
            };

            comp.energy += 10;
        }
    }

    fn handle_render(&mut self) {
        let mut layers: Vec<Vec<(Dimemsion, Buffer)>> = vec![
            vec![],
            vec![],
            vec![],
        ];
        for gen_index in self.entity_allocator.get_vec() {
            let comp = match self.render_components.get_mut(gen_index) {
                Some(comp) => comp,
                None => continue,
            };
                
            // Handling Begins

            if comp.visible {
                let tl = comp.position_tl;
                //let br = comp.position_br;
                let matrix: Buffer = comp.make_render();//get_matrix(tl, br, comp.backgroud + comp.character);
                layers[comp.layer as usize].push((tl, matrix));
            }
        }
        for layer in layers.iter().rev() {
            for matrix in layer {
                self.renderer.insert_matrix(matrix.0, &matrix.1);
            }
        }
    }

    fn handle_enemy_energy_move(&mut self) {
        for gen_index in self.entity_allocator.get_vec() {
            let ai_type: AIType = match self.enemy_ai_components.get(gen_index) {
                Some(ai_type) => ai_type.ai_type.clone(),
                None => continue,
            };
            
            let render_comp = match self.render_components.get(gen_index) {
                Some(comp) => comp,
                None => continue,
            };

            let comp = match self.energy_components.get_mut(gen_index) {
                Some(comp) => comp,
                None => continue,
            };

            let move_comp = match self.movement_components.get_mut(gen_index) {
                Some(component) => component,
                None => continue,
            };         
            if comp.energy >= 10 {
                match ai_type {
                    AIType::SimpleDown => {
                        match simple_down(render_comp.position_tl, &self.collision_buffer) {
                            (Action::DoNothing, x) => {
                                comp.energy -= x;
                            },
                            (Action::Move(dir), x) => {
                                move_comp.move_desired(dir);
                                comp.energy -= x;
                            },
                            _ => continue,
                        }
                    },
                    AIType::SimpleLeft => {
                        move_comp.move_desired(Direction::Left);
                        comp.energy -= 10;
                    },
                    AIType::RollDownPlayer => {
                        match roll_down_player(render_comp.position_tl, &self.collision_buffer, &self.player_dijk.current_generation) {
                            (Action::DoNothing, x) => {
                                comp.energy -= x;
                            },
                            (Action::Move(dir), x) => {
                                move_comp.move_desired(dir);
                                comp.energy -= x;
                            },
                            _ => continue,
                        }
                    }
                    _ => continue,
                }
            }
        }
    }

    fn handle_movement(&mut self) {
        for gen_index in self.entity_allocator.get_vec() {
            let comp = match self.movement_components.get_mut(gen_index) {
                Some(comp) => comp,
                None => continue,
            };
                
            let (desired_x, desired_y): (u16, u16) = (comp.desired_position.0, comp.desired_position.1);

            match self.render_components.get_mut(gen_index) {
                Some(render_comp) => {
                    let collided: bool = self.collision_buffer[desired_x as usize][desired_y as usize] == 1;
                    let not_moved: bool = comp.desired_position == render_comp.position_tl;
                    
                    // test if collided then stop doing anything after
                    if collided || not_moved {
                        comp.desired_position = render_comp.position_tl;
                        continue;
                    }

                    let dif_x = 0;
                    let dif_y = 0;
                    
                    render_comp.backgroud = self.renderer.input_buffer[desired_x as usize][desired_y as usize];// + render_comp.character;
                    render_comp.position_tl = (desired_x, desired_y);
                    },
                None => continue,
            }
        }
    }

    fn handle_collision(&mut self) {
        self.collision_buffer = self.empty_buffer.clone();

        let mut layers: Vec<Vec<(Dimemsion, Buffer)>> = vec![
            vec![],
            vec![],
            vec![],
        ];
        for gen_index in self.entity_allocator.get_vec() {
            let comp = match self.collision_components.get_mut(gen_index) {
                Some(comp) => comp,
                None => continue,
            };
                
            match self.render_components.get(gen_index) {
                Some(render_comp) => {
                    comp.position = render_comp.position_tl;
                    comp.matrix = render_comp.matrix.clone();
                },
                None => (),
            }
            let matrix: Buffer = comp.matrix.clone();//get_matrix(tl, br, comp.backgroud + comp.character);
            layers[comp.layer as usize].push((comp.position, matrix));
        }
        for layer in layers.iter().rev() {
            for matrix in layer {
                //self.renderer.insert_matrix(matrix.0, matrix.1.clone());
                insert_matrix(&mut self.collision_buffer, matrix.0, &matrix.1);

            }
        }
    }

    fn produce_player_dijkstra(&mut self) {
        //
    }

    fn add_entity(&mut self, components: Vec<ComponentList>) -> Entity {
        let entity = self.entity_allocator.allocate();

        for component in components {
            match component {
                ComponentList::Render(comp) => self.render_components.set(entity, comp),
                ComponentList::Collision(comp) => self.collision_components.set(entity, comp),
                ComponentList::Energy(comp) => self.energy_components.set(entity, comp),
                ComponentList::Movement(comp) => self.movement_components.set(entity, comp),
            }
        }

        return entity;
    }

    fn init_player(&mut self, position: (u16, u16)) {
        let mut comps: Vec<ComponentList> = Vec::new();
        comps.push(ComponentList::Render(RenderComponent::new(1, 1, position, get_matrix(position, position, 1), 0)));
        comps.push(ComponentList::Movement(MovementComponent::new(position)));
        comps.push(ComponentList::Collision(CollisionComponent::new(position, get_matrix(position, position, 1), 0)));

        let player_entity = Some(self.add_entity(comps));

        self.player = player_entity;
    }

    fn init_map(&mut self, percentage: f64) {
        let entity = self.entity_allocator.allocate();

        self.map_generator.randomize(percentage);
        self.map_generator.generate();

        let collision_comp = CollisionComponent::new((0, 0), self.map_generator.current_generation.clone(), 2);
        let render_comp = RenderComponent::new(2, 1, (0, 0), self.map_generator.current_generation.clone(), 2); 

        self.collision_components.set(entity, collision_comp);
        self.render_components.set(entity, render_comp);
    }

    fn init_test_enemy(&mut self, position: (u16, u16)) {
        let entity = self.entity_allocator.allocate();

        let render_comp = RenderComponent::new(3, 1, position, get_matrix(position, position, 1), 0);
        let movement_comp = MovementComponent::new(position);
        let collision_comp = CollisionComponent::new(position, get_matrix(position, position, 1), 0);
        let energy_comp = EnergyComponent::new(0);
        let enemy_ai_comp = EnemyAIComponent::new(AIType::RollDownPlayer);

        self.render_components.set(entity, render_comp);
        self.movement_components.set(entity, movement_comp);
        self.collision_components.set(entity, collision_comp);
        self.energy_components.set(entity, energy_comp);
        self.enemy_ai_components.set(entity, enemy_ai_comp);
    }

    
    fn init_borders(&mut self) {
        let top = ((1, 1), (self.renderer.view_port.0, 1));
        let left = ((1, 1), (1, self.renderer.view_port.1));
        let right = ((self.renderer.view_port.0, 1), (self.renderer.view_port.0, self.renderer.view_port.1));
        let bottom = ((1, self.renderer.view_port.1), (self.renderer.view_port.0, self.renderer.view_port.1));
        let test = ((20, 20), (30, 30));

        let borders = vec![top, bottom, left, right, test];
        // add collision comp and render comp

        for border in borders {
            let entity = self.entity_allocator.allocate();
            
            let collision_comp = CollisionComponent::new(border.0, get_matrix(border.0, border.1, 1), 0);
            let render_comp = RenderComponent::new(2, 1, border.0, get_matrix(border.0, border.1, 1), 0);
            
            self.collision_components.set(entity, collision_comp);
            self.render_components.set(entity, render_comp);
        }
    }

    fn init_field(&mut self) {
        let field: Vec<Vec<u8>> = vec![
            vec![0; 20],
        ];
        let entity = self.entity_allocator.allocate();

        let render_comp = RenderComponent::new(0, 5, (50, 10), get_matrix((50, 10), (70, 30), 1), 1);
        self.render_components.set(entity, render_comp);
    }

    fn init_background(&mut self) {
        let entity = self.entity_allocator.allocate();
        let dimensions = self.renderer.dimensions;

        let render_comp = RenderComponent::new(0, 1, (0, 0), get_matrix((0, 0), (dimensions.0, dimensions.1), 1), 2);
        self.render_components.set(entity, render_comp);
    }
}

fn get_matrix(tl: (u16, u16), br: (u16, u16), value: u8) -> Buffer {
    let length: usize;
    if br.0 == tl.0 {
        length = 1;
    } else {
        length = (br.0 - tl.0 + 1 ).into();
    }
    let height: usize;
    if br.1 == tl.1 {
        height = 1;
    } else {
        height = (br.1 - tl.1 + 1 ).into();
    }
    vec![vec![value; height]; length]
}

fn insert_matrix(buffer: &mut Buffer, location: (u16, u16), matrix: &Buffer) {
    let (column, row) : (usize, usize) = (location.0.into(), location.1.into());

    for (x, col) in matrix.iter().enumerate() {
        for (y, value) in col.iter().enumerate() {
            buffer[column + x][row + y] = *value;
        }
    }
}

fn is_event_availble() -> Result<bool> {
    poll(Duration::from_secs(0))
}