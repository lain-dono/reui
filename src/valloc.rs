use std::ops::{Index, IndexMut, Range};

pub struct VecAlloc<T>(Vec<T>);

impl<T> Index<Range<u32>> for VecAlloc<T> {
    type Output = [T];
    #[inline]
    fn index(&self, raw: Range<u32>) -> &Self::Output {
        &self.0[raw.start as usize..raw.end as usize]
    }
}

impl<T> IndexMut<Range<u32>> for VecAlloc<T> {
    #[inline]
    fn index_mut(&mut self, raw: Range<u32>) -> &mut Self::Output {
        &mut self.0[raw.start as usize..raw.end as usize]
    }
}

impl<T> AsRef<[T]> for VecAlloc<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self.0.as_ref()
    }
}

impl<T> VecAlloc<T> {
    #[inline]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    #[inline]
    pub fn push(&mut self, value: T) -> u32 {
        let start = self.0.len();
        self.0.push(value);
        start as u32
    }

    #[inline]
    pub fn alloc_with<F: FnMut() -> T>(&mut self, count: usize, f: F) -> (Range<u32>, &mut [T]) {
        let start = self.0.len();
        self.0.resize_with(start + count as usize, f);
        (
            start as u32..start as u32 + count as u32,
            &mut self.0[start..start + count],
        )
    }

    #[inline]
    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) -> Range<u32> {
        let start = self.0.len() as u32;
        self.0.extend(iter);
        let end = self.0.len() as u32;
        start..end
    }
}

impl<T: Default> VecAlloc<T> {
    #[inline]
    pub fn alloc(&mut self, count: usize) -> (Range<u32>, &mut [T]) {
        self.alloc_with(count, Default::default)
    }
}

impl<T: Copy> VecAlloc<T> {
    #[inline]
    pub fn extend_with(&mut self, src: &[T]) -> Range<u32> {
        let start = self.0.len() as u32;
        self.0.extend_from_slice(src);
        let end = self.0.len() as u32;
        start..end
    }
}
