use std::fmt;

#[derive(Copy, Clone, Debug)]
pub struct GenerationalIndex {
    pub index: usize,
    pub generation: u64,
}

impl GenerationalIndex {
    pub fn index(&self) -> usize {
        self.index
    }
}

#[derive(Copy, Clone, Debug)]
pub struct AllocatorEntry {
    is_live: bool,
    pub generation: u64,
}


#[derive(Clone, Debug)]
pub struct GenerationalIndexAllocator {
    entries: Vec<AllocatorEntry>,
    free: Vec<usize>,
}

impl GenerationalIndexAllocator {
    pub fn new(size: usize) -> GenerationalIndexAllocator {
        let free: Vec<usize> = (0..size).collect();
        let entries = vec![
            AllocatorEntry {
                is_live: false,
                generation: 0,
            }; 
            size
        ];

        GenerationalIndexAllocator {
            entries,
            free,
        }
    }

    pub fn expose_entries(&self) -> &Vec<AllocatorEntry> {
        &self.entries
    }

    pub fn get_vec(&self) -> Vec<GenerationalIndex> {
        let mut result: Vec<GenerationalIndex> = Vec::new();

        for (index, generation) in self.entries.iter().enumerate() {
            let gen_index = GenerationalIndex {
                index,
                generation: generation.generation,
            };
            if generation.is_live {
                result.push(gen_index);
            }
        }

        return result;
    }

    pub fn allocate(&mut self) -> GenerationalIndex {
        if self.free.is_empty() {
            self.free.push(self.entries.len() as usize);
        }
        
        let last_free = self.free[0];

        if last_free >= self.entries.len() {
            self.entries.push(AllocatorEntry {
                is_live: false,
                generation: 0,
            });
        }
        let mut allocator = &mut self.entries[last_free];
        allocator.generation += 1;
        allocator.is_live = true;
        
        self.free.remove(0);

        GenerationalIndex {
            index: last_free,
            generation: allocator.generation,
        }
    }

    pub fn deallocate(&mut self, index: GenerationalIndex) -> bool {
        let mut allocator = self.entries[index.index];

        if allocator.is_live && allocator.generation == index.generation {
            self.free.push(index.index);
            allocator.is_live = false;
            return true;
        } 

        return false;
    }

    pub fn is_live(&self, index: GenerationalIndex) -> bool {
        let entry = self.entries[index.index];

        return entry.is_live;
    }
}


#[derive(Copy, Clone, Debug)]
struct ArrayEntry<T> {
    value: T,
    generation: u64,
}


#[derive(Clone, Debug)]
pub struct GenerationalIndexArray<T>(Vec<Option<ArrayEntry<T>>>);
// need to build iter for T

impl<T> GenerationalIndexArray<T> {
    pub fn new() -> GenerationalIndexArray<T> {
        GenerationalIndexArray::<T> (
            Vec::new(),
        )
    }

    pub fn get_len(&self) -> usize {
        self.0.len()
    }
     
    pub fn set(&mut self, index: GenerationalIndex, value: T) {
        let entry = ArrayEntry::<T> {
            value,
            generation: index.generation,
        };
        
        if index.index >= self.0.len() {
            let difference = index.index - self.0.len();
            for i in 0..(difference + 1) {
                self.0.push(None);
            }
        }

        self.0[index.index] = Some(entry);
    }

    pub fn get(&self, index: GenerationalIndex) -> Option<&T> {
        let generation = index.generation;

        if index.index >= self.0.len() {
            return None;
        }

        let entry = match &self.0[index.index] {
            Some(entry) => entry,
            None => return None,
        };

        if entry.generation == generation {

            return Some(&entry.value);
        }

        return None;
    }

    pub fn get_mut(&mut self, index: GenerationalIndex) -> Option<&mut T> {
        let generation = index.generation;

        if index.index >= self.0.len() {
            return None;
        }

        let entry = match &mut self.0[index.index] {
            Some(entry) => entry,
            None => return None,
        };
        if entry.generation == generation {
            return Some(&mut entry.value);
        }

        return None;
    }
}