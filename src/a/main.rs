use std::time::Duration;

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
use crate::renderer::Dimemsion;
use crate::renderer::Renderer;
use crate::renderer::StyleMap;
use crate::renderer::Buffer;
mod components;
use crate::components::CollisionComponent;
use crate::components::RenderComponent;
use crate::components::MovementComponent;
use crate::components::Direction;

type EntityMap<T> = generations::GenerationalIndexArray<T>;
pub type Entity = generations::GenerationalIndex;

/*
Notes: 
kinda funkin
thinkin its the movement handler
not quite using desired_pos right

ok move working, now fields things that arent collider but visible
*/

fn main() -> Result<()> {
    let style_map: StyleMap = vec![ // add 5 for player
        '*'.on_red(),           // 0  test
        '#'.on_blue(),          // 1
        ' '.on(Color::Grey),    // 2
        ' '.on(Color::Blue),    // 3  default backgroud for now
        ' '.on(Color::Red),     // 4
        ' '.on(Color::Green),   // 5
        ' '.on(Color::Yellow),  // 6
        '@'.on(Color::Grey),    // 7
        '@'.on(Color::Blue),    // 8  default player for now
        '@'.on(Color::Red),     // 9
        '@'.on(Color::Green),   // 10
        '@'.on(Color::Yellow),  // 11
    ];

    let dimensions: Dimemsion = (101, 101);
    let view_port: Dimemsion = (80, 40);

    let mut game: GameState = GameState::new(dimensions, view_port, style_map);

    game.init_player((15, 15));
    game.init_borders();
    game.init_field();
    game.init_background();

    while game.running {
        game.handle_collision();
        game.handle_movement();
        game.handle_render();
        game.renderer.render()?;

        if is_event_availble()? {
            game.handle_input(read()?);
        }
    }

    Renderer::reset_term()?;

    Ok(())
}

struct GameState {
    // resources
    renderer: Renderer,
    collision_buffer: Buffer,
    running: bool,
    empty_buffer: Buffer,

    // ECS
    entity_allocator: GenerationalIndexAllocator,
    // for now assume every entity has render comp
    render_components: EntityMap<RenderComponent>,
    movement_components: EntityMap<MovementComponent>,
    collision_components: EntityMap<CollisionComponent>,

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
        let empty_buffer = renderer.input_buffer.clone();
        
        // ECS
        let entity_allocator = generations::GenerationalIndexAllocator::new(1);
        let render_components = EntityMap::<RenderComponent>::new();
        let movement_components = EntityMap::<MovementComponent>::new();
        let collision_components = EntityMap::<CollisionComponent>::new();
        
        GameState {
            renderer,
            collision_buffer,
            running: true,
            empty_buffer,
            entity_allocator,
            render_components,
            movement_components,
            collision_components,
            player: None,
        }
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
                    KeyCode::Right => self.move_entity(player, Direction::Right),
                    KeyCode::Left => self.move_entity(player, Direction::Left),
                    KeyCode::Up => self.move_entity(player, Direction::Up),
                    KeyCode::Down => self.move_entity(player, Direction::Down),
                    _ => return,
                }
            },
            _ => return,
        }
    }

    fn handle_render(&mut self) {
        let mut layers: Vec<Vec<(Dimemsion, Buffer)>> = vec![
            vec![],
            vec![],
            vec![],
        ];
        for gen_index in self.entity_allocator.get_vec() {
            //if self.entity_allocator.is_live(gen_index) {
                let comp = match self.render_components.get_mut(gen_index) {
                    Some(comp) => comp,
                    None => continue,
                };
                
                // Handling Begins

                if comp.visible {
                    let tl = comp.position_tl;
                    let br = comp.position_br;
                    let matrix: Buffer = get_matrix(tl, br, comp.character);
                    layers[comp.layer as usize].push((tl, matrix));
                }
        }
        for layer in layers.iter().rev() {
            for matrix in layer {
                self.renderer.insert_matrix(matrix.0, matrix.1.clone());
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
                    
                    render_comp.character = self.renderer.input_buffer[desired_x as usize][desired_y as usize] + 5;
                    render_comp.position_tl = (desired_x, desired_y);
                    render_comp.position_br = (desired_x + dif_x as u16, desired_y + dif_y as u16);

                    },
                None => continue,
            }
        }
    }

    fn handle_collision(&mut self) {
        // Ugh gotta fucking clear each time DUH
        self.collision_buffer = self.empty_buffer.clone();
        for gen_index in self.entity_allocator.get_vec() {
            //if self.entity_allocator.is_live(gen_index) {
                let comp = match self.collision_components.get_mut(gen_index) {
                    Some(comp) => comp,
                    None => continue,
                };
                
                // Handling Begins

                // Update to potential new position
                match self.render_components.get_mut(gen_index) {
                    Some(render_comp) => {
                        comp.top_left = render_comp.position_tl;
                        comp.bottom_right = render_comp.position_br;
                    },
                    None => (),
                }

                // Build and insert into collision buffer
                let matrix = get_matrix(comp.top_left, comp.bottom_right, 1);
                insert_matrix(&mut self.collision_buffer, comp.top_left, matrix);
            //}
        }
    }

    fn init_player(&mut self, position: (u16, u16)) {
        let player_entity = self.entity_allocator.allocate();

        let render_comp = RenderComponent::new(8, position, position, 0);
        let movement_comp = MovementComponent::new(position);
        let collision_comp = CollisionComponent::new(position, position);

        self.render_components.set(player_entity, render_comp);
        self.movement_components.set(player_entity, movement_comp);
        self.collision_components.set(player_entity, collision_comp);

        self.player = Some(player_entity)
    }

    fn init_borders(&mut self) {
        let top = ((1, 1), (self.renderer.dimensions.0, 1));
        let left = ((1, 1), (1, self.renderer.dimensions.1));
        let right = ((self.renderer.dimensions.0, 1), (self.renderer.dimensions.0, self.renderer.dimensions.1));
        let bottom = ((1, self.renderer.dimensions.1), (self.renderer.dimensions.0, self.renderer.dimensions.1));
        let test = ((20, 20), (30, 30));

        let borders = vec![top, bottom, left, right, test];
        // add collision comp and render comp

        for border in borders {
            let entity = self.entity_allocator.allocate();
            
            let collision_comp = CollisionComponent::new(border.0, border.1);
            let render_comp = RenderComponent::new(1, border.0, border.1, 0);
            
            self.collision_components.set(entity, collision_comp);
            self.render_components.set(entity, render_comp);
        }
    }

    fn init_field(&mut self) {
        let field: Vec<Vec<u8>> = vec![
            vec![0; 20]; 20
        ];
        let entity = self.entity_allocator.allocate();

        let render_comp = RenderComponent::new(6, (50, 10), (70, 30), 1);
        self.render_components.set(entity, render_comp);
    }

    fn init_background(&mut self) {
        let entity = self.entity_allocator.allocate();

        let render_comp = RenderComponent::new(3, (0, 0), (80, 40), 2);
        self.render_components.set(entity, render_comp);
    }
}

fn get_matrix(tl: (u16, u16), br: (u16, u16), value: u8) -> Buffer {
    let mut length: usize;
    if br.0 == tl.0 {
        length = 1;
    } else {
        length = (br.0 - tl.0 + 1 ).into();
    }
    let mut height: usize;
    if br.1 == tl.1 {
        height = 1;
    } else {
        height = (br.1 - tl.1 + 1 ).into();
    }
    vec![vec![value; height]; length]
}

fn insert_matrix(buffer: &mut Buffer, location: (u16, u16), matrix: Buffer) {
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