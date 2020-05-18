use crate::math::Transform;

#[derive(Default)]
pub struct TransformStack(Transform, Vec<Transform>);

impl TransformStack {
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Default::default(), Vec::with_capacity(capacity))
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0 = Default::default();
        self.1.clear();
    }

    #[inline]
    pub fn save_count(&mut self) -> usize {
        self.1.len()
    }

    #[inline]
    pub fn save(&mut self) {
        self.1.push(*self.last())
    }

    #[inline]
    pub fn restore(&mut self) -> bool {
        self.1.pop().is_some()
    }

    #[inline]
    pub fn transform(&self) -> Transform {
        *self.last()
    }

    #[inline]
    pub fn pre_transform(&mut self, m: Transform) {
        *self.last_mut() *= m;
    }

    #[inline(always)]
    fn last(&self) -> &Transform {
        self.1.last().unwrap_or(&self.0)
    }

    #[inline(always)]
    fn last_mut(&mut self) -> &mut Transform {
        self.1.last_mut().unwrap_or(&mut self.0)
    }
}
