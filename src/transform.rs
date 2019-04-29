use crate::context::Context;

impl Context {
    pub fn transform(&mut self, m: [f32; 6]) {
        premul(&mut self.states.last_mut().xform, &m)
    }
    pub fn reset_transform(&mut self) {
        self.states.last_mut().xform = identity();
    }
    pub fn translate(&mut self, x: f32, y: f32) {
        premul(&mut self.states.last_mut().xform, &translate(x, y));
    }
    pub fn rotate(&mut self, angle: f32) {
        premul(&mut self.states.last_mut().xform, &rotate(angle));
    }
    pub fn skew_x(&mut self, angle: f32) {
        premul(&mut self.states.last_mut().xform, &skew_x(angle));
    }
    pub fn skew_y(&mut self, angle: f32) {
        premul(&mut self.states.last_mut().xform, &skew_y(angle));
    }
    pub fn scale(&mut self, x: f32, y: f32) {
        premul(&mut self.states.last_mut().xform, &scale(x, y));
    }
    pub fn current_transform(&self) -> &[f32; 6] {
        &self.states.last().xform
    }
}

pub macro point($t: expr, $x: expr, $y: expr) {
    $x = ($x)*($t)[0] + ($y)*($t)[2] + ($t)[4];
    $y = ($x)*($t)[1] + ($y)*($t)[3] + ($t)[5];
}


/// Sets the transform to identity matrix.
pub const fn identity() -> [f32; 6] {
    [
        1.0, 0.0,
        0.0, 1.0,
        0.0, 0.0,
    ]
}

/// Sets the transform to translation matrix matrix.
pub const fn translate(tx: f32, ty: f32) -> [f32; 6] {
    [
        1.0, 0.0,
        0.0, 1.0,
        tx,  ty,
    ]
}

/// Sets the transform to scale matrix.
pub const fn scale(sx: f32, sy: f32) -> [f32; 6] {
    [
        sx, 0.0,
        0.0, sy,
        0.0, 0.0,
    ]
}

/// Sets the transform to rotate matrix.
/// Angle is specified in radians.
pub fn rotate(a: f32) -> [f32; 6] {
    let (sn, cs) = a.sin_cos();
    [
        cs,  sn,
        -sn,  cs,
        0.0, 0.0,
    ]
}

/// Sets the transform to skew-x matrix.
/// Angle is specified in radians.
pub fn skew_x(a: f32) -> [f32; 6] {
    [
        1.0, 0.0,
        a.tan(), 1.0,
        0.0, 0.0,
    ]
}

/// Sets the transform to skew-y matrix.
/// Angle is specified in radians.
pub fn skew_y(a: f32) -> [f32; 6] {
    [
        1.0, a.tan(),
        0.0, 1.0,
        0.0, 0.0,
    ]
}

/// Sets the transform to the result of multiplication of two transforms, of A = A*B.
pub fn mul(t: &mut [f32; 6], s: &[f32; 6]) {
    let t0 = t[0] * s[0] + t[1] * s[2];
    let t2 = t[2] * s[0] + t[3] * s[2];
    let t4 = t[4] * s[0] + t[5] * s[2] + s[4];
    t[1] = t[0] * s[1] + t[1] * s[3];
    t[3] = t[2] * s[1] + t[3] * s[3];
    t[5] = t[4] * s[1] + t[5] * s[3] + s[5];
    t[0] = t0;
    t[2] = t2;
    t[4] = t4;
}

/// Sets the transform to the result of multiplication of two transforms, of A = B*A.
pub fn premul(t: &mut [f32; 6], s: &[f32; 6]) {
    let mut s2 = *s;
    mul(&mut s2, t);
    *t = s2;
}

/// Sets the destination to inverse of specified transform.
/// Returns `true` if the inverse could be calculated, else `false`.
pub fn inverse_checked(inv: &mut [f32; 6], t: &[f32; 6]) -> bool {
    let t = [
        f64::from(t[0]),
        f64::from(t[1]),
        f64::from(t[2]),
        f64::from(t[3]),
        f64::from(t[4]),
        f64::from(t[5]),
    ];

    let det = t[0] * t[3] - t[2] * t[1];
    if det > -1e-6 && det < 1e-6 {
        *inv = identity();
        false
    } else {
        let invdet = 1.0 / det;
        inv[0] = (t[3] * invdet) as f32;
        inv[2] = (-t[2] * invdet) as f32;
        inv[4] = ((t[2] * t[5] - t[3] * t[4]) * invdet) as f32;
        inv[1] = (-t[1] * invdet) as f32;
        inv[3] = (t[0] * invdet) as f32;
        inv[5] = ((t[1] * t[4] - t[0] * t[5]) * invdet) as f32;
        true
    }
}

pub fn inverse(t: &[f32; 6]) -> [f32; 6] {
    let t = [
        f64::from(t[0]),
        f64::from(t[1]),
        f64::from(t[2]),
        f64::from(t[3]),
        f64::from(t[4]),
        f64::from(t[5]),
    ];

    let det = t[0] * t[3] - t[2] * t[1];
    if det > -1e-6 && det < 1e-6 {
        identity()
    } else {
        let invdet = 1.0 / det;
        let mut inv = [0f32; 6];
        inv[0] = (t[3] * invdet) as f32;
        inv[2] = (-t[2] * invdet) as f32;
        inv[4] = ((t[2] * t[5] - t[3] * t[4]) * invdet) as f32;
        inv[1] = (-t[1] * invdet) as f32;
        inv[3] = (t[0] * invdet) as f32;
        inv[5] = ((t[1] * t[4] - t[0] * t[5]) * invdet) as f32;
        inv
    }
}

pub fn transform_pt(pt: &mut [f32], t: &[f32; 6]) {
    let sx = pt[0];
    let sy = pt[1];

    pt[0] = sx*t[0] + sy*t[2] + t[4];
    pt[1] = sx*t[1] + sy*t[3] + t[5];
}

pub fn transform_point(t: &[f32; 6], sx: f32, sy: f32) -> [f32; 2] {
    [
        sx*t[0] + sy*t[2] + t[4],
        sx*t[1] + sy*t[3] + t[5],
    ]
}
