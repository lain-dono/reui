#![allow(unused_attributes)]
#![allow(clippy::many_single_char_names)]

mod images;

use crate::{
    context::{Context, Align, GlyphPosition, TextRow},
    backend::{Image, BackendGL, NFlags},
    cache::{Winding, LineJoin, LineCap},
    vg::*,
    utils::{raw_str},
};

use std::ffi::{c_void, CStr};
use std::os::raw::{c_char};

#[no_mangle] extern "C"
fn nvgDeleteGL2(_ctx: *const u8) {
}

#[no_mangle] extern "C"
fn nvgCreateGL2(flags: NFlags) -> Box<crate::context::Context> {
    Box::new(crate::context::Context::new(BackendGL::new(flags)))
}

/// Begin drawing a new frame
///
/// Calls to nanovg drawing API should be wrapped in nvgBeginFrame() & nvgEndFrame()
/// nvgBeginFrame() defines the size of the window to render to in relation currently
/// set viewport (i.e. glViewport on GL backends). Device pixel ration allows to
/// control the rendering on Hi-DPI devices.
///
/// For example, GLFW returns two dimension for an opened window: window size and
/// frame buffer size. In that case you would set windowWidth/Height to the window size
/// devicePixelRatio to: frameBufferWidth / windowWidth.
#[no_mangle] extern "C"
fn nvgBeginFrame(ctx: &mut Context, width: f32, height: f32, dpi: f32) {
    ctx.begin_frame(width, height, dpi)
}

/// Cancels drawing the current frame.
#[no_mangle] extern "C"
fn nvgCancelFrame(ctx: &mut Context) {
    ctx.cancel_frame()
}

/// Ends drawing flushing remaining render state.
#[no_mangle] extern "C"
fn nvgEndFrame(ctx: &mut Context) {
    ctx.end_frame()
}

//
// Composite operation
//
// The composite operations in NanoVG are modeled after HTML Canvas API, and
// the blend func is based on OpenGL (see corresponding manuals for more info).
// The colors in the blending state have premultiplied alpha.

/// Sets the composite operation. The op parameter should be one of NVGcompositeOperation.
#[no_mangle] extern "C"
fn nvgGlobalCompositeOperation(ctx: &mut Context, op: CompositeOp) {
    ctx.global_composite(op)
}

/// Sets the composite operation with custom pixel arithmetic. The parameters should be one of NVGblendFactor.
#[no_mangle] extern "C"
fn nvgGlobalCompositeBlendFunc(ctx: &mut Context, sfactor: BlendFactor, dfactor: BlendFactor) {
    ctx.global_blend_separate(sfactor, dfactor, sfactor, dfactor);
}

/// Sets the composite operation with custom pixel arithmetic for RGB and alpha components separately.
/// The parameters should be one of NVGblendFactor.
#[no_mangle] extern "C"
fn nvgGlobalCompositeBlendFuncSeparate(
    ctx: &mut Context,
    src_color: BlendFactor,
    dst_color: BlendFactor,
    src_alpha: BlendFactor,
    dst_alpha: BlendFactor,
) {
    ctx.global_blend_separate(src_color, dst_color, src_alpha, dst_alpha);
}

//
// Color utils
//
// Colors in NanoVG are stored as unsigned ints in ABGR format.

/// Returns a color value from red, green, blue values. Alpha will be set to 255 (1.0f).
#[no_mangle] extern "C"
fn nvgRGB(r: u8, g: u8, b: u8) -> Color {
    Color::rgb(r, g, b)
}

/// Returns a color value from red, green, blue values. Alpha will be set to 1.0f.
#[no_mangle] extern "C"
fn nvgRGBf(r: f32, g: f32, b: f32) -> Color {
    Color::rgbf(r, g, b)
}

/// Returns a color value from red, green, blue and alpha values.
#[no_mangle] extern "C"
fn nvgRGBA(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::rgba(r, g, b, a)
}

/// Returns a color value from red, green, blue and alpha values.
#[no_mangle] extern "C"
fn nvgRGBAf(r: f32, g: f32, b: f32, a: f32) -> Color {
    Color::rgbaf(r, g, b, a)
}

/// Linearly interpolates from color c0 to c1, and returns resulting color value.
#[no_mangle] extern "C"
fn nvgLerpRGBA(a: Color, b: Color, t: f32) -> Color {
    Color::lerp(a, b, t)
}

/// Sets transparency of a color value.
#[no_mangle] extern "C"
fn nvgTransRGBA(c: Color, a: u8) -> Color {
    c.trans(a)
}

/// Sets transparency of a color value.
#[no_mangle] extern "C"
fn nvgTransRGBAf(c: Color, a: f32) -> Color {
    c.transf(a)
}

/// Returns color value specified by hue, saturation and lightness.
/// HSL values are all in range [0..1], alpha will be set to 255.
#[no_mangle] extern "C"
fn nvgHSL(h: f32, s: f32, l: f32) -> Color {
    Color::hsl(h, s, l)
}

/// Returns color value specified by hue, saturation and lightness and alpha.
/// HSL values are all in range [0..1], alpha in range [0..255]
#[no_mangle] extern "C"
fn nvgHSLA(h: f32, s: f32, l: f32, a: u8) -> Color {
    Color::hsla(h, s, l, a)
}

//
// State Handling
//
// NanoVG contains state which represents how paths will be rendered.
// The state contains transform, fill and stroke styles, text and font styles,
// and scissor clipping.

/// Pushes and saves the current render state into a state stack.
/// A matching nvgRestore() must be used to restore the state.
#[no_mangle] pub extern "C"
fn nvgSave(ctx: &mut Context) {
    ctx.save();
}

/// Pops and restores current render state.
#[no_mangle] pub extern "C"
fn nvgRestore(ctx: &mut Context) {
    ctx.restore();
}

/// Resets current render state to default values. Does not affect the render state stack.
#[no_mangle] pub extern "C"
fn nvgReset(ctx: &mut Context) {
    ctx.reset();
}

//
// Render styles
//
// Fill and stroke render style can be either a solid color or a paint which is a gradient or a pattern.
// Solid color is simply defined as a color value, different kinds of paints can be created
// using nvgLinearGradient(), nvgBoxGradient(), nvgRadialGradient() and nvgImagePattern().
//
// Current render style can be saved and restored using nvgSave() and nvgRestore().

/* TODO
// Sets whether to draw antialias for nvgStroke() and nvgFill(). It's enabled by default.
void nvgShapeAntiAlias(ctx: &mut Context, int enabled);
*/

/// Sets current stroke style to a solid color.
#[no_mangle] extern "C"
fn nvgStrokeColor(ctx: &mut Context, color: Color) {
    ctx.stroke_color(color)
}

/// Sets current stroke style to a paint, which can be a one of the gradients or a pattern.
#[no_mangle] extern "C"
fn nvgStrokePaint(ctx: &mut Context, paint: Paint) {
    ctx.stroke_paint(paint)
}

/// Sets current fill style to a solid color.
#[no_mangle] extern "C"
fn nvgFillColor(ctx: &mut Context, color: Color) {
    ctx.fill_color(color)
}

/// Sets current fill style to a paint, which can be a one of the gradients or a pattern.
#[no_mangle] extern "C"
fn nvgFillPaint(ctx: &mut Context, paint: Paint) {
    ctx.fill_paint(paint)
}

/// Sets the miter limit of the stroke style.
/// Miter limit controls when a sharp corner is beveled.
#[no_mangle] extern "C"
fn nvgMiterLimit(ctx: &mut Context, limit: f32) {
    ctx.miter_limit(limit)
}

/// Sets the stroke width of the stroke style.
#[no_mangle] extern "C"
fn nvgStrokeWidth(ctx: &mut Context, size: f32) {
    ctx.stroke_width(size)
}

/// Sets how the end of the line (cap) is drawn,
/// Can be one of: NVG_BUTT (default), NVG_ROUND, NVG_SQUARE.
#[no_mangle] extern "C"
fn nvgLineCap(ctx: &mut Context, cap: LineCap) {
    ctx.line_cap(cap)
}

/// Sets how sharp path corners are drawn.
/// Can be one of NVG_MITER (default), NVG_ROUND, NVG_BEVEL.
#[no_mangle] extern "C"
fn nvgLineJoin(ctx: &mut Context, join: LineJoin) {
    ctx.line_join(join)
}

/// Sets the transparency applied to all rendered shapes.
/// Already transparent paths will get proportionally more transparent as well.
#[no_mangle] extern "C"
fn nvgGlobalAlpha(ctx: &mut Context, alpha: f32) {
    ctx.global_alpha(alpha)
}

//
// Transforms
//
// The paths, gradients, patterns and scissor region are transformed by an transformation
// matrix at the time when they are passed to the API.
// The current transformation matrix is a affine matrix:
//   [sx kx tx]
//   [ky sy ty]
//   [ 0  0  1]
// Where: sx,sy define scaling, kx,ky skewing, and tx,ty translation.
// The last row is assumed to be 0,0,1 and is not stored.
//
// Apart from nvgResetTransform(), each transformation function first creates
// specific transformation matrix and pre-multiplies the current transformation by it.
//
// Current coordinate system (transformation) can be saved and restored using nvgSave() and nvgRestore().

/// Resets current transform to a identity matrix.
#[no_mangle] extern "C"
fn nvgResetTransform(ctx: &mut Context) {
    ctx.reset_transform()
}

/// Premultiplies current coordinate system by specified matrix.
/// The parameters are interpreted as matrix as follows:
///   [a c e]
///   [b d f]
///   [0 0 1]
#[no_mangle] extern "C"
fn nvgTransform(ctx: &mut Context, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
    ctx.transform([a, b, c, d, e, f])
}

/// Translates current coordinate system.
#[no_mangle] extern "C"
fn nvgTranslate(ctx: &mut Context, x: f32, y: f32) {
    ctx.translate(x, y)
}

/// Rotates current coordinate system. Angle is specified in radians.
#[no_mangle] extern "C"
fn nvgRotate(ctx: &mut Context, angle: f32) {
    ctx.rotate(angle)
}

/// Skews the current coordinate system along X axis. Angle is specified in radians.
#[no_mangle] extern "C"
fn nvgSkewX(ctx: &mut Context, angle: f32) {
    ctx.skew_x(angle)
}

/// Skews the current coordinate system along Y axis. Angle is specified in radians.
#[no_mangle] extern "C"
fn nvgSkewY(ctx: &mut Context, angle: f32) {
    ctx.skew_y(angle)
}

/// Scales the current coordinate system.
#[no_mangle] extern "C"
fn nvgScale(ctx: &mut Context, x: f32, y: f32) {
    ctx.scale(x, y)
}

/// Stores the top part (a-f) of the current transformation matrix in to the specified buffer.
///   [a c e]
///   [b d f]
///   [0 0 1]
/// There should be space for 6 floats in the return buffer for the values a-f.
#[no_mangle] extern "C"
fn nvgCurrentTransform(ctx: &mut Context, xform: &mut [f32; 6]) {
    *xform = *ctx.current_transform();
}

// The following functions can be used to make calculations on 2x3 transformation matrices.
// A 2x3 matrix is represented as float[6].

/// Sets the transform to identity matrix.
#[no_mangle] pub extern "C"
fn nvgTransformIdentity(t: &mut [f32; 6]) {
    t[0] = 1.0; t[1] = 0.0;
    t[2] = 0.0; t[3] = 1.0;
    t[4] = 0.0; t[5] = 0.0;
}

/// Sets the transform to translation matrix matrix.
#[no_mangle] pub extern "C"
fn nvgTransformTranslate(t: &mut [f32; 6], tx: f32, ty: f32) {
    t[0] = 1.0; t[1] = 0.0;
    t[2] = 0.0; t[3] = 1.0;
    t[4] = tx; t[5] = ty;
}

/// Sets the transform to scale matrix.
#[no_mangle] pub extern "C"
fn nvgTransformScale(t: &mut [f32; 6], sx: f32, sy: f32) {
    t[0] = sx; t[1] = 0.0;
    t[2] = 0.0; t[3] = sy;
    t[4] = 0.0; t[5] = 0.0;
}

/// Sets the transform to rotate matrix. Angle is specified in radians.
#[no_mangle] pub extern "C"
fn nvgTransformRotate(t: &mut [f32; 6], a: f32) {
    let (sn, cs) = a.sin_cos();
    t[0] = cs; t[1] = sn;
    t[2] = -sn; t[3] = cs;
    t[4] = 0.0; t[5] = 0.0;
}

/// Sets the transform to skew-x matrix. Angle is specified in radians.
#[no_mangle] pub extern "C"
fn nvgTransformSkewX(t: &mut [f32; 6], a: f32) {
    t[0] = 1.0; t[1] = 0.0;
    t[2] = a.tan(); t[3] = 1.0;
    t[4] = 0.0; t[5] = 0.0;
}

/// Sets the transform to skew-y matrix. Angle is specified in radians.
#[no_mangle] pub extern "C"
fn nvgTransformSkewY(t: &mut [f32; 6], a: f32) {
    t[0] = 1.0; t[1] = a.tan();
    t[2] = 0.0; t[3] = 1.0;
    t[4] = 0.0; t[5] = 0.0;
}

/// Sets the transform to the result of multiplication of two transforms, of A = A*B.
#[no_mangle] pub extern "C"
fn nvgTransformMultiply(t: &mut [f32; 6], s: &[f32; 6]) {
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
#[no_mangle] pub extern "C"
fn nvgTransformPremultiply(dst: &mut [f32; 6], src: &[f32; 6]) {
    let mut s2 = *src;
    nvgTransformMultiply(&mut s2, dst);
    *dst = s2;
}

/// Sets the destination to inverse of specified transform.
/// Returns 1 if the inverse could be calculated, else 0.
#[no_mangle] pub extern "C"
fn nvgTransformInverse(inv: &mut [f32; 6], t: &[f32; 6]) -> bool {
    crate::transform::inverse_checked(inv, t)
}

/// Transform a point by given transform.
#[no_mangle] pub extern "C"
fn nvgTransformPoint(dx: &mut f32, dy: &mut f32, t: &[f32; 6], sx: f32, sy: f32) {
    *dx = sx*t[0] + sy*t[2] + t[4];
    *dy = sx*t[1] + sy*t[3] + t[5];
}


// Converts degrees to radians and vice versa.
#[no_mangle] extern "C"
fn nvgDegToRad(deg: f32) -> f32 { crate::vg::utils::deg2rad(deg) }
#[no_mangle] extern "C"
fn nvgRadToDeg(rad: f32) -> f32 { crate::vg::utils::rad2deg(rad) }

//
// Paints
//
// NanoVG supports four types of paints: linear gradient, box gradient, radial gradient and image pattern.
// These can be used as paints for strokes and fills.

/// Creates and returns a linear gradient. Parameters (sx,sy)-(ex,ey) specify the start and end coordinates
/// of the linear gradient, icol specifies the start color and ocol the end color.
/// The gradient is transformed by the current transform when it is passed to nvgFillPaint() or nvgStrokePaint().
#[no_mangle] extern "C"
fn nvgLinearGradient(_ctx: &mut c_void, sx: f32, sy: f32, ex: f32, ey: f32, icol: Color, ocol: Color) -> Paint {
    Paint::linear_gradient(sx, sy, ex, ey, icol, ocol)
}

/// Creates and returns a box gradient. Box gradient is a feathered rounded rectangle, it is useful for rendering
/// drop shadows or highlights for boxes. Parameters (x,y) define the top-left corner of the rectangle,
/// (w,h) define the size of the rectangle, r defines the corner radius, and f feather. Feather defines how blurry
/// the border of the rectangle is. Parameter icol specifies the inner color and ocol the outer color of the gradient.
/// The gradient is transformed by the current transform when it is passed to nvgFillPaint() or nvgStrokePaint().
#[no_mangle] extern "C"
fn nvgRadialGradient(_ctx: &mut c_void, cx: f32, cy: f32, inr: f32, outr: f32, icol: Color, ocol: Color) -> Paint {
    Paint::radial_gradient(cx, cy, inr, outr, icol, ocol)
}

/// Creates and returns a radial gradient. Parameters (cx,cy) specify the center, inr and outr specify
/// the inner and outer radius of the gradient, icol specifies the start color and ocol the end color.
/// The gradient is transformed by the current transform when it is passed to nvgFillPaint() or nvgStrokePaint().

#[no_mangle] extern "C"
fn nvgBoxGradient(_ctx: &mut c_void, x: f32, y: f32, w: f32, h: f32, r: f32, f: f32, icol: Color, ocol: Color) -> Paint {
    Paint::box_gradient(x, y, w, h, r, f, icol, ocol)
}

/// Creates and returns an image patter. Parameters (ox,oy) specify the left-top location of the image pattern,
/// (ex,ey) the size of one image, angle rotation around the top-left corner, image is handle to the image to render.
/// The gradient is transformed by the current transform when it is passed to nvgFillPaint() or nvgStrokePaint().
#[no_mangle] extern "C"
fn nvgImagePattern(_ctx: &mut c_void, cx: f32, cy: f32, w: f32, h: f32, angle: f32, image: Image, alpha: f32) -> Paint {
    Paint::image_pattern(cx, cy, w, h, angle, image, alpha)
}


//
// Scissoring
//
// Scissoring allows you to clip the rendering into a rectangle. This is useful for various
// user interface cases like rendering a text edit or a timeline.

/// Sets the current scissor rectangle.
/// The scissor rectangle is transformed by the current transform.
#[no_mangle] pub extern "C"
fn nvgScissor(ctx: &mut Context, x: f32, y: f32, w: f32, h: f32) {
    ctx.scissor(x, y, w, h);
}


/// Intersects current scissor rectangle with the specified rectangle.
/// The scissor rectangle is transformed by the current transform.
///
/// NOTE: in case the rotation of previous scissor rect differs from
/// the current one, the intersection will be done between the specified
/// rectangle and the previous scissor rectangle transformed in the current
/// transform space. The resulting shape is always rectangle.
#[no_mangle] pub extern "C"
fn nvgIntersectScissor(ctx: &mut Context, x: f32, y: f32, w: f32, h: f32) {
    ctx.intersect_scissor(x, y, w, h);
}

/// Reset and disables scissoring.
#[no_mangle] pub extern "C"
fn nvgResetScissor(ctx: &mut Context) {
    ctx.reset_scissor();
}

//
// Paths
//
// Drawing a new shape starts with nvgBeginPath(), it clears all the currently defined paths.
// Then you define one or more paths and sub-paths which describe the shape. The are functions
// to draw common shapes like rectangles and circles, and lower level step-by-step functions,
// which allow to define a path curve by curve.
//
// NanoVG uses even-odd fill rule to draw the shapes. Solid shapes should have counter clockwise
// winding and holes should have counter clockwise order. To specify winding of a path you can
// call nvgPathWinding(). This is useful especially for the common shapes, which are drawn CCW.
//
// Finally you can fill the path using current fill style by calling nvgFill(), and stroke it
// with current stroke style by calling nvgStroke().
//
// The curve segments and sub-paths are transformed by the current transform.

/// Clears the current path and sub-paths.
#[no_mangle] pub extern "C"
fn nvgBeginPath(ctx: &mut Context) {
    ctx.begin_path();
}

/// Starts new sub-path with specified point as first point.
#[no_mangle] pub extern "C"
fn nvgMoveTo(ctx: &mut Context, x: f32, y: f32) {
    ctx.move_to(x, y);
}

/// Adds line segment from the last point in the path to the specified point.
#[no_mangle] pub extern "C"
fn nvgLineTo(ctx: &mut Context, x: f32, y: f32) {
    ctx.line_to(x, y);
}

/// Adds cubic bezier segment from last point in the path via two control points to the specified point.
#[no_mangle] pub extern "C"
fn nvgBezierTo(ctx: &mut Context, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
    ctx.bezier_to(c1x, c1y, c2x, c2y, x, y);
}

/// Adds quadratic bezier segment from last point in the path via a control point to the specified point.
#[no_mangle] pub extern "C"
fn nvgQuadTo(ctx: &mut Context, cx: f32, cy: f32, x: f32, y: f32) {
    ctx.quad_to(cx, cy, x, y);
}

/// Adds an arc segment at the corner defined by the last path point, and two specified points.
#[no_mangle] pub extern "C"
fn nvgArcTo(ctx: &mut Context, x1: f32, y1: f32, x2: f32, y2: f32, radius: f32) {
    ctx.arc_to(x1, y1, x2, y2, radius);
}

/// Closes current sub-path with a line segment.
#[no_mangle] pub extern "C"
fn nvgClosePath(ctx: &mut Context) {
    ctx.close_path();
}

/// Sets the current sub-path winding, see NVGwinding and NVGsolidity.
#[no_mangle] pub extern "C"
fn nvgPathWinding(ctx: &mut Context, dir: Winding) {
    ctx.path_winding(dir);
}

/// Creates new circle arc shaped sub-path. The arc center is at cx,cy, the arc radius is r,
/// and the arc is drawn from angle a0 to a1, and swept in direction dir (NVG_CCW, or NVG_CW).
/// Angles are specified in radians.
#[no_mangle] pub extern "C"
fn nvgArc(ctx: &mut Context, cx: f32, cy: f32, r: f32, a0: f32, a1: f32, dir: Winding) {
    ctx.arc(cx, cy, r, a0, a1, dir);
}

/// Creates new rectangle shaped sub-path.
#[no_mangle] pub extern "C"
fn nvgRect(ctx: &mut Context, x: f32, y: f32, w: f32, h: f32) {
    ctx.rect(x, y, w, h);
}

/// Creates new rounded rectangle shaped sub-path.
#[no_mangle] pub extern "C"
fn nvgRoundedRect(ctx: &mut Context, x: f32, y: f32, w: f32, h: f32, r: f32) {
    ctx.rrect(x, y, w, h, r);
}

/// Creates new rounded rectangle shaped sub-path with varying radii for each corner.
#[no_mangle] pub extern "C"
fn nvgRoundedRectVarying(
    ctx: &mut Context,
    x: f32, y: f32, w: f32, h: f32,
    radTopLeft: f32, radTopRight: f32, radBottomRight: f32, radBottomLeft: f32,
) {
    ctx.rrect_varying(x, y, w, h, radTopLeft, radTopRight, radBottomRight, radBottomLeft);
}

/// Creates new ellipse shaped sub-path.
#[no_mangle] pub extern "C"
fn nvgEllipse(ctx: &mut Context, cx: f32, cy: f32, rx: f32, ry: f32) {
    ctx.ellipse(cx, cy, rx, ry);
}

/// Creates new circle shaped sub-path.
#[no_mangle] pub extern "C"
fn nvgCircle(ctx: &mut Context, cx: f32, cy: f32, r: f32) {
    ctx.circle(cx, cy, r);
}

/// Fills the current path with current fill style.
#[no_mangle] pub extern "C"
fn nvgFill(ctx: &mut Context) {
    ctx.fill();
}

/// Fills the current path with current stroke style.
#[no_mangle] pub extern "C"
fn nvgStroke(ctx: &mut Context) {
    ctx.stroke();
}

/*
fn nvgDebugDumpPathCache(ctx: &mut Context) {
    println!("Dumping {} cached paths", ctx.cache.npaths);
    for i in 0..ctx.cache.npaths; i++) {
        let path = &ctx.cache.paths[i];
        println!(" - Path [}", i);
        if (path.nfill) {
            println!("   - fill: {}", path.nfill);
            for j in 0..path.nfill {
                println!("{}\t{}", path.fill[j].x, path.fill[j].y);
            }
        }
        if path.nstroke {
            println!("   - stroke: %d", path.nstroke);
            for j in 0..path.nstroke {
                println!("{}\t{}", path.stroke[j].x, path.stroke[j].y);
            }
        }
    }
}
*/


//
// Text
//
// NanoVG allows you to load .ttf files and use the font to render text.
//
// The appearance of the text can be defined by setting the current text style
// and by specifying the fill color. Common text and font settings such as
// font size, letter spacing and text align are supported. Font blur allows you
// to create simple text effects such as drop shadows.
//
// At render time the font face can be set based on the font handles or name.
//
// Font measure functions return values in local space, the calculations are
// carried in the same resolution as the final rendering. This is done because
// the text glyph positions are snapped to the nearest pixels sharp rendering.
//
// The local space means that values are not rotated or scale as per the current
// transformation. For example if you set font size to 12, which would mean that
// line height is 16, then regardless of the current scaling and rotation, the
// returned line height is always 16. Some measures may vary because of the scaling
// since aforementioned pixel snapping.
//
// While this may sound a little odd, the setup allows you to always render the
// same way regardless of scaling. I.e. following works regardless of scaling:
//
//        const char* txt = "Text me up.";
//        nvgTextBounds(vg, x,y, txt, NULL, bounds);
//        nvgBeginPath(vg);
//        nvgRoundedRect(vg, bounds[0],bounds[1], bounds[2]-bounds[0], bounds[3]-bounds[1]);
//        nvgFill(vg);
//
// NOTE: currently only solid color fill is supported for text.

// Creates font by loading it from the disk from specified file name.
// Returns handle to the font.
#[no_mangle] extern "C"
fn nvgCreateFont(ctx: &mut Context, name: *const c_char, path: *const c_char) -> i32 {
    let name = unsafe { CStr::from_ptr(name).to_string_lossy() };
    let path = unsafe { CStr::from_ptr(path).to_string_lossy() };
    ctx.create_font(&name, &path)
}

// Creates font by loading it from the specified memory chunk.
// Returns handle to the font.
#[no_mangle] extern "C"
fn nvgCreateFontMem(ctx: &mut Context, name: *const c_char, data: *mut u8, ndata: i32, free_data: i32) -> i32 {
    let name = unsafe { CStr::from_ptr(name).to_string_lossy() };
    ctx.fs.add_font_mem(&name, data, ndata, free_data)
}


// Finds a loaded font of specified name, and returns handle to it, or -1 if the font is not found.
#[no_mangle] extern "C"
fn nvgFindFont(ctx: &mut Context, name: *const u8) -> i32 {
    if name.is_null() {
        -1
    } else {
        ctx.fs.font_by_name(name)
    }
}

// Adds a fallback font by handle.
#[no_mangle] extern "C"
fn nvgAddFallbackFontId(ctx: &mut Context, base: i32, fallback: i32) -> i32 {
    if base == -1 || fallback == -1 {
        0
    } else {
        ctx.fs.add_fallback_font(base, fallback)
    }
}

// Adds a fallback font by name.
#[no_mangle] extern "C"
fn nvgAddFallbackFont(ctx: &mut Context, base: *const u8, fallback: *const u8) -> i32 {
    let base = nvgFindFont(ctx, base);
    let fallback = nvgFindFont(ctx, fallback);
    nvgAddFallbackFontId(ctx, base, fallback)
}

// Sets the font size of current text style.
#[no_mangle] extern "C"
fn nvgFontSize(ctx: &mut Context, size: f32) {
    ctx.states.last_mut().font_size = size;
}

// Sets the blur of current text style.
#[no_mangle] extern "C"
fn nvgFontBlur(ctx: &mut Context, blur: f32) {
    ctx.states.last_mut().font_blur = blur;
}

// Sets the letter spacing of current text style.
#[no_mangle] extern "C"
fn nvgTextLetterSpacing(ctx: &mut Context, spacing: f32) {
    ctx.states.last_mut().letter_spacing = spacing;
}

// Sets the proportional line height of current text style. The line height is specified as multiple of font size.
#[no_mangle] extern "C"
fn nvgTextLineHeight(ctx: &mut Context, line_height: f32) {
    ctx.states.last_mut().line_height = line_height;
}

// Sets the text align of current text style, see NVGalign for options.
#[no_mangle] extern "C"
fn nvgTextAlign(ctx: &mut Context, align: Align) {
    ctx.states.last_mut().text_align = align;
}

// Sets the font face based on specified id of current text style.
#[no_mangle] extern "C"
fn nvgFontFaceId(ctx: &mut Context, font_id: i32) {
    ctx.states.last_mut().font_id = font_id;
}

// Sets the font face based on specified name of current text style.
#[no_mangle] extern "C"
fn nvgFontFace(ctx: &mut Context, font: *const u8) {
    ctx.states.last_mut().font_id = ctx.fs.font_by_name(font);
}



// Draws text string at specified location. If end is specified only the sub-string up to the end is drawn.
#[no_mangle] unsafe extern "C"
fn nvgText(ctx: &mut Context, x: f32, y: f32, start: *const u8, end: *const u8) -> f32 {
    ctx.text_raw(x, y, start, end)
}

// Draws multi-line text string at specified location wrapped at the specified width. If end is specified only the sub-string up to the end is drawn.
// White space is stripped at the beginning of the rows, the text is split at word boundaries or when new-line characters are encountered.
// Words longer than the max width are slit at nearest character (i.e. no hyphenation).
#[no_mangle] unsafe extern "C"
fn nvgTextBox(ctx: &mut Context, x: f32, y: f32, break_row_width: f32, start: *const u8, end: *const u8) {
    ctx.text_box(x, y, break_row_width, raw_str(start, end))
}

// Measures the specified text string. Parameter bounds should be a pointer to float[4],
// if the bounding box of the text should be returned. The bounds value are [xmin,ymin, xmax,ymax]
// Returns the horizontal advance of the measured text (i.e. where the next character should drawn).
// Measured values are returned in local coordinate space.
#[no_mangle] unsafe extern "C"
fn nvgTextBounds(ctx: &mut Context, x: f32, y: f32, start: *const u8, end: *const u8, bounds: *mut [f32; 4]) -> f32 {
    let (w, b) = ctx.text_bounds(x, y, raw_str(start, end));
    if !bounds.is_null() {
        *bounds = b;
    }
    w
}

// Measures the specified multi-text string. Parameter bounds should be a pointer to float[4],
// if the bounding box of the text should be returned. The bounds value are [xmin,ymin, xmax,ymax]
// Measured values are returned in local coordinate space.
#[no_mangle] unsafe extern "C"
fn nvgTextBoxBounds(
    ctx: &mut Context, x: f32, y: f32, break_row_width: f32,
    start: *const u8, end: *const u8, bounds: *mut [f32; 4],
) {
    if !bounds.is_null() {
        *bounds = ctx.text_box_bounds(x, y, break_row_width, raw_str(start, end));
    }
}

// Calculates the glyph x positions of the specified text. If end is specified only the sub-string will be used.
// Measured values are returned in local coordinate space.
#[no_mangle] unsafe extern "C"
fn nvgTextGlyphPositions(
    ctx: &mut Context, x: f32, y: f32,
    start: *const u8, end: *const u8,
    positions: *mut GlyphPosition,
    max_positions: i32,
) -> usize {
    let text = raw_str(start, end);
    let positions = std::slice::from_raw_parts_mut(positions, max_positions as usize);
    ctx.text_glyph_positions(x, y, text, positions).len()
}

// Returns the vertical metrics based on the current text style.
// Measured values are returned in local coordinate space.
#[no_mangle] unsafe extern "C"
fn nvgTextMetrics(ctx: &mut Context, ascender: *mut f32, descender: *mut f32, lineh: *mut f32) {
    if let Some(m) = ctx.text_metrics() {
        if !ascender.is_null() {
            *ascender = m.ascender;
        }
        if !descender.is_null() {
            *descender = m.descender;
        }
        if !lineh.is_null() {
            *lineh = m.line_height;
        }
    }
}

// Breaks the specified text into lines. If end is specified only the sub-string will be used.
// White space is stripped at the beginning of the rows, the text is split at word boundaries or when new-line characters are encountered.
// Words longer than the max width are slit at nearest character (i.e. no hyphenation).
#[no_mangle] unsafe extern "C"
fn nvgTextBreakLines(
    ctx: &mut Context, start: *const u8, end: *const u8,
    break_row_width: f32, rows: *mut TextRow, max_rows: usize,
) -> usize {
    let text = raw_str(start, end);
    let rows = std::slice::from_raw_parts_mut(rows, max_rows);
    ctx.text_break_lines(text, break_row_width, rows).len()
}