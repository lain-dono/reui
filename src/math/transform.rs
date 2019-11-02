#[derive(Clone, Copy)]
pub struct Transform {
    pub re: f32, // ssin
    pub im: f32, // scos
    pub tx: f32,
    pub ty: f32,
}

impl Transform {
    pub const IDENTITY: Self = Self { re: 0.0, im: 1.0, tx: 0.0, ty: 0.0 };

    #[inline]
    pub const fn identity() -> Self {
        Self::IDENTITY
    }

    #[inline]
    pub const fn new(re: f32, im: f32, tx: f32, ty: f32) -> Self {
        Self { re, im, tx, ty }
    }

    #[inline]
    pub const fn from_sin_cos(ssin: f32, scos: f32) -> Self {
        Self { re: ssin, im: scos, .. Self::IDENTITY }
    }

    #[inline]
    pub const fn translation(tx: f32, ty: f32) -> Self {
        Self { tx, ty, .. Self::IDENTITY }
    }

    #[inline]
    pub const fn scale(scale: f32) -> Self {
        Self { im: scale, .. Self::IDENTITY }
    }

    #[inline]
    pub fn rotation(angle: f32) -> Self {
        let (re, im) = angle.sin_cos();
        Self { re, im, .. Self::IDENTITY }
    }

    #[inline]
    #[doc(hidden)]
    pub const fn create_translation(tx: f32, ty: f32) -> Self {
        Self { tx, ty, .. Self::IDENTITY }
    }

    pub fn compose<A: Into<[f32; 2]>, T: Into<[f32; 2]>>(
        rotation: f32,
        scale: f32,
        anchor: A,
        translate: T,
    ) -> Self {
        let [anchor_x, anchor_y] = anchor.into();
        let [translate_x, translate_y] = translate.into();

        let (sn, cs) = rotation.sin_cos();

        let re = sn * scale;
        let im = cs * scale;
        let tx = translate_x + -im * anchor_x + re * anchor_y;
        let ty = translate_y + -re * anchor_x - im * anchor_y;

        Self { im, re, tx, ty }
    }

    #[inline]
    pub fn angle(&self) -> f32 {
        self.re.atan2(self.im)
    }

    #[inline]
    pub fn scaling(&self) -> f32 {
        (self.im * self.im + self.re * self.re).sqrt()
    }

    #[inline]
    pub fn apply(&self, [x, y]: [f32; 2]) -> [f32; 2] {
        let x = self.tx + x * self.im - y * self.re;
        let y = self.ty + y * self.im + x * self.re;
        [x, y]
    }

    #[inline]
    pub fn cast_apply<T>(&self, p: T) -> [f32; 2]
        where T: From<[f32; 2]> + Into<[f32; 2]>
    {
        let [x, y] = p.into();
        let x = self.tx + x * self.im - y * self.re;
        let y = self.ty + y * self.im + x * self.re;
        [x, y].into()
    }

    /*
    #[inline]
    pub fn to_matrix2(&self) -> [f32; 4] {
        let r = self.re;
        let i = self.im;
        [r, -i, i, r]
    }
    */

    #[inline]
    pub fn append(&self, m: &Self) -> Self {
        Self {
            re: self.re * m.re - self.im * m.im,
            im: self.im * m.re + self.re * m.im,
            tx: m.tx * self.im - m.ty * self.re + self.tx,
            ty: m.ty * self.im + m.tx * self.re + self.ty,
        }
    }
}

#[test]
fn transform() {
    let angle: f32 = 0.032;
    let (sn, cs) = angle.sin_cos();
    let tr = Transform::from_sin_cos(sn, cs);

    assert_eq!(angle, tr.angle());
    assert_eq!(1.0, tr.scaling());
}