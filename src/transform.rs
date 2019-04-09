use crate::context::Context;

impl Context {
    pub fn transform(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
        premul(&mut self.states.last_mut().xform, &[
            a, b, c, d, e, f,
        ]);
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
        t[0] as f64,
        t[1] as f64,
        t[2] as f64,
        t[3] as f64,
        t[4] as f64,
        t[5] as f64,
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
        t[0] as f64,
        t[1] as f64,
        t[2] as f64,
        t[3] as f64,
        t[4] as f64,
        t[5] as f64,
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


/*
void nvgTransformIdentity(float* t)
{
    t[0] = 1.0f; t[1] = 0.0f;
    t[2] = 0.0f; t[3] = 1.0f;
    t[4] = 0.0f; t[5] = 0.0f;
}

void nvgTransformTranslate(float* t, float tx, float ty)
{
    t[0] = 1.0f; t[1] = 0.0f;
    t[2] = 0.0f; t[3] = 1.0f;
    t[4] = tx; t[5] = ty;
}

void nvgTransformScale(float* t, float sx, float sy)
{
    t[0] = sx; t[1] = 0.0f;
    t[2] = 0.0f; t[3] = sy;
    t[4] = 0.0f; t[5] = 0.0f;
}

void nvgTransformRotate(float* t, float a)
{
    float cs = nvg__cosf(a), sn = nvg__sinf(a);
    t[0] = cs; t[1] = sn;
    t[2] = -sn; t[3] = cs;
    t[4] = 0.0f; t[5] = 0.0f;
}

void nvgTransformSkewX(float* t, float a)
{
    t[0] = 1.0f; t[1] = 0.0f;
    t[2] = nvg__tanf(a); t[3] = 1.0f;
    t[4] = 0.0f; t[5] = 0.0f;
}

void nvgTransformSkewY(float* t, float a)
{
    t[0] = 1.0f; t[1] = nvg__tanf(a);
    t[2] = 0.0f; t[3] = 1.0f;
    t[4] = 0.0f; t[5] = 0.0f;
}

void nvgTransformMultiply(float* t, const float* s)
{
    float t0 = t[0] * s[0] + t[1] * s[2];
    float t2 = t[2] * s[0] + t[3] * s[2];
    float t4 = t[4] * s[0] + t[5] * s[2] + s[4];
    t[1] = t[0] * s[1] + t[1] * s[3];
    t[3] = t[2] * s[1] + t[3] * s[3];
    t[5] = t[4] * s[1] + t[5] * s[3] + s[5];
    t[0] = t0;
    t[2] = t2;
    t[4] = t4;
}

void nvgTransformPremultiply(float* t, const float* s)
{
    float s2[6];
    memcpy(s2, s, sizeof(float)*6);
    nvgTransformMultiply(s2, t);
    memcpy(t, s2, sizeof(float)*6);
}

int nvgTransformInverse(float* inv, const float* t)
{
    double invdet, det = (double)t[0] * t[3] - (double)t[2] * t[1];
    if (det > -1e-6 && det < 1e-6) {
            nvgTransformIdentity(inv);
            return 0;
    }
    invdet = 1.0 / det;
    inv[0] = (float)(t[3] * invdet);
    inv[2] = (float)(-t[2] * invdet);
    inv[4] = (float)(((double)t[2] * t[5] - (double)t[3] * t[4]) * invdet);
    inv[1] = (float)(-t[1] * invdet);
    inv[3] = (float)(t[0] * invdet);
    inv[5] = (float)(((double)t[1] * t[4] - (double)t[0] * t[5]) * invdet);
    return 1;
}
*/

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
