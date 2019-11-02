#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Transform {
    pub re: f32, // cos + scale
    pub im: f32, // sin + scale
    pub tx: f32,
    pub ty: f32,
}

impl Default for Transform {
    #[inline]
    fn default() -> Self { Self::IDENTITY }
}

impl Transform {
    pub const IDENTITY: Self = Self { re: 1.0, im: 0.0, tx: 0.0, ty: 0.0 };

    #[inline]
    pub const fn identity() -> Self { Self::IDENTITY }

    #[inline]
    pub const fn new(re: f32, im: f32, tx: f32, ty: f32) -> Self {
        Self { re, im, tx, ty }
    }

    #[inline]
    pub const fn from_sin_cos(im: f32, re: f32) -> Self {
        Self { re, im, .. Self::IDENTITY }
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
        let (im, re) = angle.sin_cos();
        Self { re, im, .. Self::IDENTITY }
    }

    #[inline]
    #[doc(hidden)]
    pub const fn create_translation(tx: f32, ty: f32) -> Self {
        Self { tx, ty, .. Self::IDENTITY }
    }

    /*
    pub fn compose_simple<T: Into<[f32; 2]> + Copy>(
        rotation: f32,
        translate: T,
    ) -> Self {
        Self::compose(rotation, 1.0, translate, [0.0, 0.0])
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
    */

    #[inline]
    pub fn angle(&self) -> f32 {
        self.im.atan2(self.re)
    }

    #[inline]
    pub fn scaling(&self) -> f32 {
        (self.im * self.im + self.re * self.re).sqrt()
    }

    #[inline]
    pub fn apply(&self, [x, y]: [f32; 2]) -> [f32; 2] {
        let Self { re: cos, im: sin, tx, ty } = self;
        [
            tx + (x * cos) - (y * sin),
            ty + (x * sin) + (y * cos),
        ]
    }

    #[inline]
    pub fn cast_apply<T: From<[f32; 2]> + Into<[f32; 2]>>(&self, p: T) -> T {
        self.apply(p.into()).into()
    }

    #[inline]
    pub fn to_matrix2(&self) -> [f32; 4] {
        let r = self.re;
        let i = self.im;
        [r, -i, i, r]
    }

    #[inline]
    pub fn to_gl(&self) -> [f32; 12] {
        let Self { re, im, tx, ty} = *self;
        [
            re, -im, 0.0, 0.0,
            im, re, 0.0, 0.0,
            tx, ty, 1.0, 0.0,
        ]
    }

    #[inline]
    pub fn append(&self, m: &Self) -> Self {
        let re = (self.re * m.re - self.im * m.im).sqrt();
        let im = (self.re * m.im + self.im * m.re).sqrt();
        Self {
            re, im,

            //tx: m.tx * self.im - m.ty * self.re + self.tx,
            //ty: m.tx * self.re + m.ty * self.im + self.ty,

            tx: m.tx * self.re - m.ty * self.im + self.tx,
            ty: m.tx * self.im + m.ty * self.re + self.ty,
        }
    }
}

#[test]
fn rotation() {
    #![allow(clippy::float_cmp)]

    fn eq(a: [f32; 2], b: [f32; 2]) {
        use euclid::approxeq::ApproxEq;
        println!("\teq {:?} {:?}", a, b);
        let eps = 1e-5;
        assert!(a[0].approx_eq_eps(&b[0], &eps) && a[1].approx_eq_eps(&b[1], &eps))
    }

    let angle: f32 = 0.032;
    let (sn, cs) = angle.sin_cos();
    let tr = Transform::from_sin_cos(sn, cs);

    assert_eq!(tr.angle(), angle);
    assert_eq!(tr.scaling(), 1.0);

    use std::f32::consts::{PI, FRAC_PI_2};

    let tr = Transform::rotation(FRAC_PI_2);
    println!("1: {:?}", tr);
    eq(tr.apply([0.0, 0.0]), [0.0, 0.0]);
    eq(tr.apply([1.0, 0.0]), [0.0, 1.0]);

    let tr = Transform::rotation(-FRAC_PI_2);
    println!("3: {:?}", tr);
    eq(tr.apply([1.0, 0.0]), [0.0, -1.0]);

    let tr = Transform::rotation(PI);
    println!("4: {:?}", tr);
    eq(tr.apply([1.0, 0.0]), [-1.0, 0.0]);

    let tr = Transform::rotation(PI*2.0);
    println!("4: {:?}", tr);
    eq(tr.apply([1.0, 0.0]), [1.0, 0.0]);
}

#[test]
fn translation() {
    #![allow(clippy::float_cmp)]

    let tr = Transform::translation(1.0, 2.0);
    assert_eq!(tr.apply([0.0, 0.0]), [1.0, 2.0]);

    let tr = Transform::translation(1.0, 2.0).append(&Transform::IDENTITY);
    assert_eq!(tr.apply([0.0, 0.0]), [1.0, 2.0]);

    let tr = Transform::IDENTITY.append(&Transform::translation(1.0, 2.0));
    assert_eq!(tr.apply([0.0, 0.0]), [1.0, 2.0]);

    let tr = Transform::translation(1.0, 0.0).append(&Transform::translation(0.0, 2.0));
    assert_eq!(tr.apply([0.0, 0.0]), [1.0, 2.0]);

    let tr = Transform::translation(0.0, 2.0).append(&Transform::translation(1.0, 0.0));
    assert_eq!(tr.apply([0.0, 0.0]), [1.0, 2.0]);
}