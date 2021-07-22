mod generations;

pub type Entity = generations::GenerationalIndex;

// Vec<Option<ArrayEntry<T>>>
type EntityMap<T> = generations::GenerationalIndexArray<T>;

pub struct GameState {
    // assests
    //renderer: Renderer,

    entity_allocator: generations::GenerationalIndexAllocator,
    render_components: EntityMap<u64/*RenderComponent*/>,

    player: Option<Entity>,
}

impl GameState {
    pub fn new() -> GameState {
        let entity_allocator = generations::GenerationalIndexAllocator::new(1);

        let render_components = EntityMap::<u64>::new();

        GameState {
            entity_allocator,
            render_components,
            player: None,
        }
    }

    pub fn test_alloc(&mut self) {
        let entity: Entity = self.entity_allocator.allocate();

        self.render_components.set(entity, 8);
    }

    pub fn test_dealloc(&mut self, entity: Entity) {
        let success = self.entity_allocator.deallocate(entity);
        let index = entity.index();

        /*match self.render_components.get(entity) {
            Some(_) => self.render_components.0[index] = None,
            None => return;
        };*/
    }

    pub fn print_it(&self) {
        println!("Entity Allocator:");
        println!("{:?} \n", self.entity_allocator);

        println!("Render Components:");
        println!("{:?} \n", self.render_components);

        println!("Playe:");
        println!("{:?} \n", self.player);
    }
}
