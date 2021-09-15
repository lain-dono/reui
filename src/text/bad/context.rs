use {
    crate::{
        geom::Offset,
        text::{
            //atlas::Atlas,
            face::{FaceCollection, FaceId},
            shaper::Shaper,
            {Error, Paint, Paragraph, TextLayout},
        },
        {Canvas, Transform},
    },
    fnv::FnvHashMap,
    std::{ffi::OsStr, fs, path::Path as FilePath},
    ttf_parser::GlyphId,
};

// This padding is an empty border around the glyph’s pixels but inside the
// sampled area (texture coordinates) for the quad in render_atlas().
const GLYPH_PADDING: u32 = 1;

// We add an additional margin of 1 pixel outside of the sampled area,
// to deal with the linear interpolation of texels at the edge of that area
// which mixes in the texels just outside of the edge.
// This manifests as noise around the glyph, outside of the padding.
const GLYPH_MARGIN: u32 = 1;

const TEXTURE_SIZE: usize = 512;

fn quantize(a: f32, d: f32) -> f32 {
    (a / d + 0.5).trunc() * d
}

pub type ImageId = usize;

pub struct FontTexture {
    //atlas: Atlas,
    image_id: ImageId,
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum RenderMode {
    Fill,
    Stroke,
}

impl Default for RenderMode {
    fn default() -> Self {
        Self::Fill
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct RenderedGlyphId {
    glyph_id: u32,
    face_id: FaceId,
    size: u32,
    line_width: u32,
    render_mode: RenderMode,
    subpixel_location: u8,
}

/*
impl RenderedGlyphId {
    fn new(
        glyph_id: u32,
        face_id: FaceId,
        font_size: f32,
        line_width: f32,
        mode: RenderMode,
        subpixel_location: u8,
    ) -> Self {
        Self {
            glyph_id,
            face_id,
            size: (font_size * 10.0).trunc() as u32,
            line_width: (line_width * 10.0).trunc() as u32,
            render_mode: mode,
            subpixel_location,
        }
    }
}
*/

#[derive(Copy, Clone, Debug)]
pub struct RenderedGlyph {
    texture_index: usize,
    width: u32,
    height: u32,
    bearing_y: i32,
    atlas_x: u32,
    atlas_y: u32,
    padding: u32,
}

pub struct TextContext {
    pub fonts: FaceCollection,
    pub textures: Vec<FontTexture>,
    pub rendered_glyphs: FnvHashMap<RenderedGlyphId, RenderedGlyph>,
    pub shaper: Shaper,
}

impl Default for TextContext {
    fn default() -> Self {
        Self {
            fonts: FaceCollection::default(),
            textures: Vec::new(),
            rendered_glyphs: FnvHashMap::default(),
            shaper: Shaper::default(),
        }
    }
}

impl TextContext {
    /// # Errors
    pub fn add_font_dir<T: AsRef<FilePath>>(&mut self, path: T) -> Result<Vec<FaceId>, Error> {
        let path = path.as_ref();
        let mut fonts = Vec::new();

        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let path = entry?.path();

                if path.is_dir() {
                    self.add_font_dir(&path)?;
                } else if let Some("ttf") = path.extension().and_then(OsStr::to_str) {
                    self.clear_caches();
                    let data = std::fs::read(path)?;
                    fonts.push(self.fonts.add_mem(data, 0)?);
                }
            }
        }

        Ok(fonts)
    }

    /// # Errors
    pub fn add_font<T: AsRef<FilePath>>(&mut self, path: T) -> Result<FaceId, Error> {
        self.clear_caches();
        let data = std::fs::read(path)?;
        self.fonts.add_mem(data, 0)
    }

    /// # Errors
    pub fn add_font_mem(&mut self, data: &[u8]) -> Result<FaceId, Error> {
        self.clear_caches();
        self.fonts.add_mem(data, 0)
    }

    fn clear_caches(&mut self) {
        self.shaper.words_cache.clear();
    }

    #[cfg(feature = "debug_inspector")]
    pub fn debug_inspector_get_textures(&self) -> impl Iterator<Item = ImageId> + '_ {
        self.textures.iter().map(|t| t.image_id)
    }
}

// Renderer

#[derive(Clone, Debug)]
pub(crate) struct DrawCmd {
    pub image_id: ImageId,
    pub quads: Vec<Quad>,
}

#[derive(Copy, Clone, Default, Debug)]
pub(crate) struct Quad {
    pub x0: f32,
    pub y0: f32,

    pub x1: f32,
    pub y1: f32,

    pub s0: f32,
    pub t0: f32,

    pub s1: f32,
    pub t1: f32,
}

impl TextContext {
    /*
    pub(crate) fn render_atlas(
        &mut self,
        canvas: &mut Canvas,
        text_layout: &TextMetrics,
        paint: &Paint,
        mode: RenderMode,
    ) -> Result<Vec<DrawCmd>, Error> {
        let mut cmd_map = FnvHashMap::default();

        let line_width_offset = if mode == RenderMode::Stroke {
            (paint.line_width / 2.0).ceil()
        } else {
            0.0
        };

        //let initial_render_target = canvas.current_render_target;

        for glyph in &text_layout.glyphs {
            let subpixel_location = quantize(glyph.x.fract(), 0.1) * 10.0;

            let id = RenderedGlyphId::new(
                glyph.codepoint,
                glyph.font_id,
                paint.font_size,
                paint.line_width,
                mode,
                subpixel_location as u8,
            );

            if !self.rendered_glyphs.contains_key(&id) {
                let glyph = self.render_glyph(canvas, paint, mode, &glyph)?;

                self.rendered_glyphs.insert(id, glyph);
            }

            let rendered = self.rendered_glyphs.get(&id).unwrap();

            if let Some(texture) = self.textures.get(rendered.texture_index) {
                let image_id = texture.image_id;
                let size = texture.atlas.size();
                let it_w = 1.0 / size.0 as f32;
                let it_h = 1.0 / size.1 as f32;

                let cmd = cmd_map
                    .entry(rendered.texture_index)
                    .or_insert_with(|| DrawCmd {
                        image_id,
                        quads: Vec::new(),
                    });

                let mut q = Quad::default();

                q.x0 = glyph.x.trunc() - line_width_offset - GLYPH_PADDING as f32;
                q.y0 = (glyph.y + glyph.bearing_y).round()
                    - rendered.bearing_y as f32
                    - line_width_offset
                    - GLYPH_PADDING as f32;
                q.x1 = q.x0 + rendered.width as f32;
                q.y1 = q.y0 + rendered.height as f32;

                q.s0 = rendered.atlas_x as f32 * it_w;
                q.t0 = rendered.atlas_y as f32 * it_h;
                q.s1 = (rendered.atlas_x + rendered.width) as f32 * it_w;
                q.t1 = (rendered.atlas_y + rendered.height) as f32 * it_h;

                cmd.quads.push(q);
            }
        }

        //canvas.set_render_target(initial_render_target);

        Ok(cmd_map.drain().map(|(_, cmd)| cmd).collect())
    }
    */

    /*
    fn render_glyph(
        &mut self,
        canvas: &mut Canvas,
        paint: &Paint,
        mode: RenderMode,
        glyph: &ShapedGlyph,
    ) -> Result<RenderedGlyph, Error> {
        let padding = GLYPH_PADDING + GLYPH_MARGIN;

        let line_width = if mode == RenderMode::Stroke {
            paint.line_width
        } else {
            0.0
        };

        let line_width_offset = (line_width / 2.0).ceil();

        let width = glyph.width.ceil() as u32 + (line_width_offset * 2.0) as u32 + padding * 2;
        let height = glyph.height.ceil() as u32 + (line_width_offset * 2.0) as u32 + padding * 2;

        let (dst_index, dst_image_id, (dst_x, dst_y)) =
            self.find_texture_or_alloc(canvas, width as usize, height as usize)?;

        // render glyph to image
        canvas.save();
        //canvas.reset();

        let (mut path, scale) = {
            let font = self.font_mut(glyph.font_id).ok_or(Error::NotFound)?;
            let scale = font.scale(paint.font_size);

            let path = if let Some(font_glyph) = font.glyph(glyph.codepoint as u16) {
                font_glyph.path.clone()
            } else {
                Path::new()
            };

            (path, scale)
        };

        let rendered_bearing_y = glyph.bearing_y.round();
        let x_quant = quantize(glyph.x.fract(), 0.1);
        let x = dst_x as f32 - glyph.bearing_x + line_width_offset + padding as f32 + x_quant;
        let y = TEXTURE_SIZE as f32
            - dst_y as f32
            - rendered_bearing_y
            - line_width_offset
            - padding as f32;

        canvas.translate(x, y);

        /*
        canvas.set_render_target(RenderTarget::Image(dst_image_id));
        canvas.clear_rect(
            dst_x as u32,
            TEXTURE_SIZE as u32 - dst_y as u32 - height as u32,
            width as u32,
            height as u32,
            Color::black(),
        );
        */

        let factor = 1.0 / 8.0;

        let mut mask_paint = Paint::color(Color::rgbf(factor, factor, factor));
        mask_paint.set_fill_rule(FillRule::EvenOdd);
        mask_paint.set_anti_alias(false);

        if mode == RenderMode::Stroke {
            mask_paint.line_width = line_width / scale;
        }

        // 4x
        // let points = [
        //     (-3.0/8.0, 1.0/8.0),
        //     (1.0/8.0, 3.0/8.0),
        //     (3.0/8.0, -1.0/8.0),
        //     (-1.0/8.0, -3.0/8.0),
        // ];

        // 8x
        let points = [
            (-7.0 / 16.0, -1.0 / 16.0),
            (-1.0 / 16.0, -5.0 / 16.0),
            (3.0 / 16.0, -7.0 / 16.0),
            (5.0 / 16.0, -3.0 / 16.0),
            (7.0 / 16.0, 1.0 / 16.0),
            (1.0 / 16.0, 5.0 / 16.0),
            (-3.0 / 16.0, 7.0 / 16.0),
            (-5.0 / 16.0, 3.0 / 16.0),
        ];

        for point in &points {
            canvas.save();
            canvas.translate(point.0, point.1);
            canvas.scale(scale);
            canvas.draw_path(&path, mask_paint);
            canvas.restore();
        }

        canvas.restore();

        Ok(RenderedGlyph {
            width: width - 2 * GLYPH_MARGIN,
            height: height - 2 * GLYPH_MARGIN,
            bearing_y: rendered_bearing_y as i32,
            atlas_x: dst_x as u32 + GLYPH_MARGIN,
            atlas_y: dst_y as u32 + GLYPH_MARGIN,
            texture_index: dst_index,
            padding: padding - GLYPH_MARGIN,
        })
    }
    */

    /*
    // Returns (texture index, image id, glyph padding box)
    fn find_texture_or_alloc(
        &mut self,
        canvas: &mut Canvas,
        width: usize,
        height: usize,
    ) -> Result<(usize, ImageId, (usize, usize)), Error> {
        // Find a free location in one of the atlases
        let mut textures = self.textures.iter_mut().enumerate();
        let mut texture_search_result = textures.find_map(|(index, texture)| {
            texture
                .atlas
                .add_rect(width, height)
                .map(|loc| (index, texture.image_id, loc))
        });

        if texture_search_result.is_none() {
            // All atlases are exausted and a new one must be created
            let mut atlas = Atlas::new(TEXTURE_SIZE, TEXTURE_SIZE);

            let loc = atlas
                .add_rect(width, height)
                .ok_or(Error::FontSizeTooLargeForAtlas)?;

            // Using PixelFormat::Gray8 works perfectly and takes less VRAM.
            // We keep Rgba8 for now because it might be useful for sub-pixel
            // anti-aliasing (ClearType®), and the atlas debug display is much
            // clearer with different colors.
            let info = ImageInfo::new(
                ImageFlags::empty(),
                atlas.size().0,
                atlas.size().1,
                PixelFormat::Rgba8,
            );
            let image_id = canvas.images.alloc(&mut canvas.renderer, info)?;

            #[cfg(feature = "debug_inspector")]
            if cfg!(debug_assertions) {
                // Fill the texture with red pixels only in debug builds.
                if let Ok(size) = canvas.image_size(image_id) {
                    canvas.save();
                    canvas.reset();
                    canvas.set_render_target(RenderTarget::Image(image_id));
                    canvas.clear_rect(
                        0,
                        0,
                        size.0 as u32,
                        size.1 as u32,
                        Color::rgb(255, 0, 0), // Shown as white if using Gray8.
                    );
                    canvas.restore();
                }
            }

            self.textures.push(FontTexture { atlas, image_id });

            let index = self.textures.len() - 1;
            texture_search_result = Some((index, image_id, loc));
        }

        texture_search_result.ok_or(Error::Unknown)
    }
    */

    /// # Errors
    pub fn render_direct(
        &mut self,
        canvas: &mut Canvas,
        layout: &TextLayout,
        paint: &Paint,
    ) -> Result<(), Error> {
        render_direct(&mut self.fonts, canvas, layout, paint)
    }
}

/// # Errors
pub fn render_direct(
    fonts: &mut FaceCollection,
    canvas: &mut Canvas,
    layout: &TextLayout,
    paint: &Paint,
) -> Result<(), Error> {
    let invscale = canvas.dpi().recip();

    /*
    //let mut paint = paint.clone();
    //mode: RenderMode,
    if let PaintingStyle::FillNonZero = paint.style {
        paint.style = PaintingStyle::FillEvenOdd
    }
    */

    //let mut scaled = false;

    for glyph in &layout.glyphs {
        let (path, scale) = {
            let font = fonts.get_mut(glyph.face_id).ok_or(Error::NotFound)?;
            let scale = font.scale(paint.font_size);

            let path = if let Some(glyph) = font.glyph(GlyphId(glyph.glyph as u16)) {
                &glyph.path
            } else {
                continue;
            };

            (path, scale)
        };

        /*
        if mode == RenderMode::Stroke && !scaled {
            paint.line_width /= scale;
            scaled = true;
        }
        */

        let tx = (glyph.position.x - glyph.bearing.x) * invscale;
        let ty = (glyph.position.y + glyph.bearing.y) * invscale;

        let transform = Transform::compose(tx, ty, 0.0, scale * invscale);

        let mut paint = crate::Paint::fill(paint.color);
        paint.style = crate::PaintingStyle::FillEvenOdd;
        canvas.draw_path(path.transform_iter(transform), paint);
    }

    Ok(())
}

/*
pub fn draw_text(
    canvas: &mut Canvas,
    x: f32,
    y: f32,
    text: &str,
    mut paint: Paint,
    render_mode: RenderMode,
) -> Result<TextMetrics, Error> {
    let transform = self.state().transform;
    let scale = self.font_scale() * self.device_px_ratio;
    let invscale = 1.0 / scale;

    self.transform_text_paint(&mut paint);

    let mut layout = text::shape(
        x * scale,
        y * scale,
        &mut self.text_context,
        &paint,
        text,
        None,
    )?;

    //let layout = self.layout_text(x, y, text, paint)?;

    // TODO: Early out if text is outside the canvas bounds, or maybe even check for each character in layout.

    if paint.font_size > 92.0 {
        text::render_direct(self, &layout, &paint, render_mode, invscale)?;
    } else {
        let cmds = text::render_atlas(self, &layout, &paint, render_mode)?;

        for cmd in &cmds {
            let mut verts = Vec::with_capacity(cmd.quads.len() * 6);

            for quad in &cmd.quads {
                let (p0, p1) = transform.transform_point(quad.x0 * invscale, quad.y0 * invscale);
                let (p2, p3) = transform.transform_point(quad.x1 * invscale, quad.y0 * invscale);
                let (p4, p5) = transform.transform_point(quad.x1 * invscale, quad.y1 * invscale);
                let (p6, p7) = transform.transform_point(quad.x0 * invscale, quad.y1 * invscale);

                verts.push(Vertex::new(p0, p1, quad.s0, quad.t0));
                verts.push(Vertex::new(p4, p5, quad.s1, quad.t1));
                verts.push(Vertex::new(p2, p3, quad.s1, quad.t0));
                verts.push(Vertex::new(p0, p1, quad.s0, quad.t0));
                verts.push(Vertex::new(p6, p7, quad.s0, quad.t1));
                verts.push(Vertex::new(p4, p5, quad.s1, quad.t1));
            }

            paint.set_alpha_mask(Some(cmd.image_id));

            // Apply global alpha
            paint.mul_alpha(self.state().alpha);

            self.render_triangles(&verts, &paint);
        }
    }

    layout.scale(invscale);

    Ok(layout)
}
*/

/*
pub fn measure_text<S: AsRef<str>>(
    context: &mut TextContext,
    x: f32,
    y: f32,
    text: S,
    mut paint: Paint,
) -> Result<TextMetrics, Error> {
    self.transform_text_paint(&mut paint);

    let text = text.as_ref();
    let scale = self.font_scale() * self.device_px_ratio;
    let invscale = 1.0 / scale;

    let mut layout = context.shape(x * scale, y * scale, &paint, text, None)?;
    layout.scale(invscale);

    Ok(layout)
}
*/

impl TextContext {
    /// Returns information on how the provided text will be drawn with the specified paint.
    /// # Errors
    pub fn measure_text<S: AsRef<str>>(
        &mut self,
        cursor: Offset,
        text: S,
        paint: &Paint,
        font_scale: f32,
        device_px_ratio: f32,
    ) -> Result<TextLayout, Error> {
        let scale = font_scale * device_px_ratio;

        let mut layout = self.shaper.shape(
            /*cursor * scale,*/ paint,
            text.as_ref(),
            None,
            &mut self.fonts,
        )?;

        layout.scale(scale.recip());

        Ok(layout)
    }
}
