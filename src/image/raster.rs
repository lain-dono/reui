use {
    super::atlas::{self, ImageAtlas},
    std::collections::{HashMap, HashSet},
    std::sync::Arc,
};

#[derive(Debug)]
pub struct Image(u64, Arc<Data>);

#[derive(Debug)]
pub struct Data {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

#[derive(Debug)]
pub enum Memory {
    Host(Data),
    Device(atlas::Entry),
}

impl Memory {
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            Memory::Host(image) => (image.width, image.height),
            Memory::Device(entry) => entry.size(),
        }
    }
}

#[derive(Default, Debug)]
pub struct Cache {
    map: HashMap<u64, Memory>,
    hits: HashSet<u64>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            hits: HashSet::new(),
        }
    }

    pub fn load(&mut self, id: u64, image: Data) -> &mut Memory {
        let hits = &mut self.hits;
        self.map
            .entry(id)
            .and_modify(|_| {
                let _ = hits.insert(id);
            })
            .or_insert_with(|| {
                let _ = hits.insert(id);
                Memory::Host(image)
            })
    }

    pub fn upload(
        &mut self,
        id: u64,
        image: Data,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        atlas: &mut ImageAtlas,
    ) -> Option<&atlas::Entry> {
        let memory = self.load(id, image);

        if let Memory::Host(image) = memory {
            let entry = atlas.upload(image.width, image.height, &image.pixels, device, encoder)?;
            *memory = Memory::Device(entry);
        }

        match memory {
            Memory::Device(allocation) => Some(allocation),
            Memory::Host(_) => None,
        }
    }

    pub fn trim(&mut self, atlas: &mut ImageAtlas) {
        let hits = &self.hits;

        self.map.retain(|k, memory| {
            let retain = hits.contains(k);
            if !retain {
                if let Memory::Device(entry) = memory {
                    atlas.remove(entry);
                }
            }
            retain
        });

        self.hits.clear();
    }
}
