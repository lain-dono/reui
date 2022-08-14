//#![allow(clippy::cast_possible_wrap)]

mod region {
    #[derive(Clone, Copy, Debug)]
    pub struct AllocId(u32);

    #[derive(Debug)]
    pub struct Region {
        pub id: AllocId,
        pub bounds: guillotiere::Rectangle,
    }

    impl Region {
        pub fn id(&self) -> AllocId {
            self.id
        }

        pub fn position(&self) -> (u32, u32) {
            (self.bounds.min.x as u32, self.bounds.min.y as u32)
        }

        pub fn size(&self) -> (u32, u32) {
            let size = self.bounds.size();
            (size.width as u32, size.height as u32)
        }

        pub fn width(&self) -> u32 {
            self.bounds.width() as u32
        }
        pub fn height(&self) -> u32 {
            self.bounds.height() as u32
        }
    }

    impl Region {
        pub(crate) fn from_etagire(alloc: etagere::Allocation) -> Self {
            Self {
                id: AllocId(alloc.id.serialize()),
                bounds: alloc.rectangle,
            }
        }

        pub(crate) fn from_guillotiere(alloc: guillotiere::Allocation) -> Self {
            Self {
                id: AllocId(alloc.id.serialize()),
                bounds: alloc.rectangle,
            }
        }
    }

    pub trait RegionAllocator {
        fn new(width: u32, height: u32) -> Self;

        fn is_empty(&self) -> bool;

        fn clear(&mut self);

        fn allocate(&mut self, width: u32, height: u32) -> Option<Region>;

        fn deallocate(&mut self, id: AllocId);
    }

    pub struct Guillotiere(pub guillotiere::AtlasAllocator);

    impl RegionAllocator for Guillotiere {
        fn new(width: u32, height: u32) -> Self {
            Self(guillotiere::AtlasAllocator::new(guillotiere::size2(
                width as i32,
                height as i32,
            )))
        }

        fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        fn clear(&mut self) {
            self.0.clear();
        }

        fn allocate(&mut self, width: u32, height: u32) -> Option<Region> {
            self.0
                .allocate(guillotiere::size2(width as i32, height as i32))
                .map(Region::from_guillotiere)
        }

        fn deallocate(&mut self, id: AllocId) {
            self.0.deallocate(guillotiere::AllocId::deserialize(id.0));
        }
    }

    pub struct Etagere(pub etagere::BucketedAtlasAllocator);

    impl RegionAllocator for Etagere {
        fn new(width: u32, height: u32) -> Self {
            Self(etagere::BucketedAtlasAllocator::new(etagere::size2(
                width as i32,
                height as i32,
            )))
        }
        fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        fn clear(&mut self) {
            self.0.clear();
        }

        fn allocate(&mut self, width: u32, height: u32) -> Option<Region> {
            self.0
                .allocate(etagere::size2(width as i32, height as i32))
                .map(Region::from_etagire)
        }

        fn deallocate(&mut self, id: AllocId) {
            self.0.deallocate(etagere::AllocId::deserialize(id.0));
        }
    }
}

pub use self::region::{Etagere, Guillotiere, Region, RegionAllocator};
use std::num::NonZeroU32;

#[derive(Debug)]
pub enum Allocation {
    Partial { layer: usize, region: Region },
    Full { layer: usize },
}

impl Allocation {
    pub fn position(&self) -> (u32, u32) {
        match self {
            Allocation::Partial { region, .. } => region.position(),
            Allocation::Full { .. } => (0, 0),
        }
    }

    pub fn size(&self) -> (u32, u32) {
        match self {
            Allocation::Partial { region, .. } => region.size(),
            Allocation::Full { .. } => (SIZE, SIZE),
        }
    }

    pub fn layer(&self) -> usize {
        match self {
            Allocation::Partial { layer, .. } | Allocation::Full { layer } => *layer,
        }
    }

    pub fn width(&self) -> u32 {
        match self {
            Allocation::Partial { region, .. } => region.width(),
            Allocation::Full { .. } => SIZE,
        }
    }

    pub fn height(&self) -> u32 {
        match self {
            Allocation::Partial { region, .. } => region.height(),
            Allocation::Full { .. } => SIZE,
        }
    }
}

#[derive(Debug)]
pub enum Entry {
    Contiguous(Allocation),
    Fragmented {
        width: u32,
        height: u32,
        fragments: Vec<Fragment>,
    },
}

impl Entry {
    pub fn size(&self) -> (u32, u32) {
        match self {
            Entry::Contiguous(allocation) => allocation.size(),
            Entry::Fragmented { width, height, .. } => (*width, *height),
        }
    }
}

#[derive(Debug)]
pub struct Fragment {
    pub position: (u32, u32),
    pub allocation: Allocation,
}

pub struct Allocator<A: RegionAllocator> {
    raw: A,
    allocations: usize,
}

impl<A: RegionAllocator> std::fmt::Debug for Allocator<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Allocator")
    }
}

impl<A: RegionAllocator> Default for Allocator<A> {
    fn default() -> Self {
        Self::new(SIZE)
    }
}

impl<A: RegionAllocator> Allocator<A> {
    fn new(size: u32) -> Self {
        Self {
            raw: A::new(size, size),
            allocations: 0,
        }
    }

    fn allocate(&mut self, width: u32, height: u32) -> Option<Region> {
        let allocation = self.raw.allocate(width, height)?;
        self.allocations += 1;
        Some(allocation)
    }

    fn deallocate(&mut self, region: &Region) {
        self.raw.deallocate(region.id);
        self.allocations = self.allocations.saturating_sub(1);
    }

    fn is_empty(&self) -> bool {
        self.allocations == 0
    }
}

pub const SIZE: u32 = 2048;

#[derive(Debug)]
enum Layer<A: RegionAllocator> {
    Empty,
    Busy(Allocator<A>),
    Full,
}

pub type ImageAtlas = Atlas<Guillotiere, 4>;
pub type GlyphAtlas = Atlas<Etagere, 1>;

pub struct Atlas<A: RegionAllocator, const BYTES_PER_PIXEL: u32> {
    texture: wgpu::Texture,
    texture_view: wgpu::TextureView,
    layers: Vec<Layer<A>>,
}

impl<A: RegionAllocator, const BYTES_PER_PIXEL: u32> Atlas<A, BYTES_PER_PIXEL> {
    pub fn new(device: &wgpu::Device) -> Self {
        let texture = Self::create_texture_bgra(device, 1);

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..wgpu::TextureViewDescriptor::default()
        });

        Self {
            texture,
            texture_view,
            layers: vec![Layer::Empty],
        }
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.texture_view
    }

    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    pub fn upload(
        &mut self,
        width: u32,
        height: u32,
        data: &[u8],
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Option<Entry> {
        use wgpu::util::DeviceExt;

        let entry = {
            let current_size = self.layers.len();
            let entry = self.allocate(width, height)?;

            // We grow the internal texture after allocating if necessary
            let new_layers = self.layers.len() - current_size;
            self.grow(new_layers, device, encoder);

            entry
        };

        //log::info!("Allocated atlas entry: {:?}", entry);

        // It is a webgpu requirement that:
        //   BufferCopyView.layout.bytes_per_row % wgpu::COPY_BYTES_PER_ROW_ALIGNMENT == 0
        // So we calculate padded_width by rounding width up to the next
        // multiple of wgpu::COPY_BYTES_PER_ROW_ALIGNMENT.
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let stride = BYTES_PER_PIXEL * width;
        let padding = (align - stride % align) % align;
        let padded_width = (stride + padding) as usize;
        let padded_data_size = padded_width * height as usize;

        let mut padded_data = vec![0; padded_data_size];

        for row in 0..height as usize {
            let offset = row * padded_width;

            padded_data[offset..offset + stride as usize]
                .copy_from_slice(&data[row * stride as usize..(row + 1) * stride as usize]);
        }

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("reui::image staging buffer"),
            contents: &padded_data,
            usage: wgpu::BufferUsages::COPY_SRC,
        });

        match &entry {
            Entry::Contiguous(allocation) => {
                self.upload_allocation(&buffer, width, height, padding, 0, allocation, encoder);
            }
            Entry::Fragmented { fragments, .. } => {
                for fragment in fragments {
                    let (x, y) = fragment.position;
                    let offset = (y * padded_width as u32 + BYTES_PER_PIXEL * x) as usize;

                    self.upload_allocation(
                        &buffer,
                        width,
                        height,
                        padding,
                        offset,
                        &fragment.allocation,
                        encoder,
                    );
                }
            }
        }

        //log::info!("Current atlas: {:?}", self);

        Some(entry)
    }

    pub fn remove(&mut self, entry: &Entry) {
        //log::info!("Removing atlas entry: {:?}", entry);

        match entry {
            Entry::Contiguous(allocation) => self.deallocate(allocation),
            Entry::Fragmented { fragments, .. } => {
                for fragment in fragments {
                    self.deallocate(&fragment.allocation);
                }
            }
        }
    }

    fn allocate(&mut self, width: u32, height: u32) -> Option<Entry> {
        // Allocate one layer if texture fits perfectly
        if width == SIZE && height == SIZE {
            let mut empty_layers = self
                .layers
                .iter_mut()
                .enumerate()
                .filter(|(_, layer)| matches!(layer, Layer::Empty));

            if let Some((i, layer)) = empty_layers.next() {
                *layer = Layer::Full;
                Some(Entry::Contiguous(Allocation::Full { layer: i }))
            } else {
                self.layers.push(Layer::Full);
                Some(Entry::Contiguous(Allocation::Full {
                    layer: self.layers.len() - 1,
                }))
            }

        // Split big textures across multiple layers
        } else if width > SIZE || height > SIZE {
            let mut fragments = Vec::new();
            let mut y = 0;

            while y < height {
                let height = std::cmp::min(height - y, SIZE);
                let mut x = 0;

                while x < width {
                    let width = std::cmp::min(width - x, SIZE);

                    let allocation = self.allocate(width, height)?;

                    if let Entry::Contiguous(allocation) = allocation {
                        fragments.push(Fragment {
                            position: (x, y),
                            allocation,
                        });
                    }

                    x += width;
                }

                y += height;
            }

            Some(Entry::Fragmented {
                width,
                height,
                fragments,
            })
        } else {
            // Try allocating on an existing layer
            for (index, layer) in self.layers.iter_mut().enumerate() {
                match layer {
                    Layer::Empty => {
                        let mut allocator = Allocator::default();
                        if let Some(region) = allocator.allocate(width, height) {
                            *layer = Layer::Busy(allocator);
                            return Some(Entry::Contiguous(Allocation::Partial {
                                region,
                                layer: index,
                            }));
                        }
                    }
                    Layer::Busy(allocator) => {
                        if let Some(region) = allocator.allocate(width, height) {
                            return Some(Entry::Contiguous(Allocation::Partial {
                                region,
                                layer: index,
                            }));
                        }
                    }
                    Layer::Full => {}
                }
            }

            // Create new layer with atlas allocator
            let mut allocator = Allocator::default();
            allocator.allocate(width, height).map(|region| {
                self.layers.push(Layer::Busy(allocator));
                Entry::Contiguous(Allocation::Partial {
                    region,
                    layer: self.layers.len() - 1,
                })
            })
        }
    }

    fn deallocate(&mut self, allocation: &Allocation) {
        //log::info!("Deallocating atlas: {:?}", allocation);

        match allocation {
            Allocation::Full { layer } => self.layers[*layer] = Layer::Empty,
            Allocation::Partial { layer, region } => {
                let layer = &mut self.layers[*layer];

                if let Layer::Busy(allocator) = layer {
                    allocator.deallocate(region);

                    if allocator.is_empty() {
                        *layer = Layer::Empty;
                    }
                }
            }
        }
    }

    fn upload_allocation(
        &mut self,
        buffer: &wgpu::Buffer,
        width: u32,
        height: u32,
        padding: u32,
        offset: usize,
        allocation: &Allocation,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let (x, y) = allocation.position();
        let layer = allocation.layer();

        encoder.copy_buffer_to_texture(
            wgpu::ImageCopyBuffer {
                buffer,
                layout: wgpu::ImageDataLayout {
                    offset: offset as u64,
                    bytes_per_row: NonZeroU32::new(BYTES_PER_PIXEL * width + padding),
                    rows_per_image: NonZeroU32::new(height),
                },
            },
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x,
                    y,
                    z: layer as u32,
                },
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: allocation.width(),
                height: allocation.height(),
                depth_or_array_layers: 1,
            },
        );
    }

    fn grow(&mut self, amount: usize, device: &wgpu::Device, encoder: &mut wgpu::CommandEncoder) {
        if amount == 0 {
            return;
        }

        let new_texture = Self::create_texture_bgra(device, self.layers.len() as u32);

        let amount_to_copy = self.layers.len() - amount;

        for (i, layer) in self.layers.iter_mut().take(amount_to_copy).enumerate() {
            if matches!(layer, Layer::Empty) {
                continue;
            }

            let z = i as u32;

            encoder.copy_texture_to_texture(
                wgpu::ImageCopyTexture {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: 0, y: 0, z },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::ImageCopyTexture {
                    texture: &new_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: 0, y: 0, z },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: SIZE,
                    height: SIZE,
                    depth_or_array_layers: 1,
                },
            );
        }

        self.texture = new_texture;
        self.texture_view = self.texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..wgpu::TextureViewDescriptor::default()
        });
    }

    fn create_texture_bgra(device: &wgpu::Device, layers: u32) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("reui::image texture atlas"),
            size: wgpu::Extent3d {
                width: SIZE,
                height: SIZE,
                depth_or_array_layers: layers,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
        })
    }
}
