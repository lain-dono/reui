use std::ptr::null;
use super::gl::{self, types::{GLint, GLuint}};

pub struct Shader {
    prog: GLuint,
    //frag: GLuint,
    //vert: GLuint,

    loc_viewsize: GLint,
    loc_tex: GLint,
    loc_frag: GLint,
}

impl Shader {
    pub fn new() -> Self {
        let (vshader, fshader) = (VERT.as_ptr() as *const i8, FRAG.as_ptr() as *const i8);

        unsafe {
            let prog = gl::CreateProgram();
            let vert = gl::CreateShader(gl::VERTEX_SHADER);
            let frag = gl::CreateShader(gl::FRAGMENT_SHADER);
            //str[2] = vshader;
            gl::ShaderSource(vert, 1, &vshader, null());
            //str[2] = fshader;
            gl::ShaderSource(frag, 1, &fshader, null());

            gl::CompileShader(vert);
            /*
            let mut status = 0;
            glGetShaderiv(vert, GL_COMPILE_STATUS, &status);
            assert!(status == 1);
            */

            gl::CompileShader(frag);
            /*
            glGetShaderiv(frag, GL_COMPILE_STATUS, &status);
            assert!(status == 1);
            */

            gl::AttachShader(prog, vert);
            gl::AttachShader(prog, frag);

            gl::BindAttribLocation(prog, 0, b"vertex\0".as_ptr() as *const i8);
            gl::BindAttribLocation(prog, 1, b"tcoord\0".as_ptr() as *const i8);

            gl::LinkProgram(prog);
            /*
            gl::GetProgramiv(prog, GL_LINK_STATUS, &status);
            assert!(status == 1);
            */

            Self {
                prog,
                //vert,
                //frag,

                loc_viewsize: gl::GetUniformLocation(prog, b"viewSize\0".as_ptr() as *const i8),
                loc_tex:      gl::GetUniformLocation(prog, b"tex\0".as_ptr() as *const i8),
                loc_frag:     gl::GetUniformLocation(prog, b"frag\0".as_ptr() as *const i8),
            }
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::UseProgram(self.prog);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::UseProgram(0);
        }
    }

    pub fn bind_frag(&self, array: &[f32; 11*4 + 1]) {
        unsafe {
            gl::Uniform4fv(self.loc_frag, 11, &(array[0]));
        }
    }

    pub fn bind_view(&self, view: *const [f32; 2]) {
        unsafe {
            gl::Uniform1i(self.loc_tex, 0);
            gl::Uniform2fv(self.loc_viewsize, 1, view as *const f32);
        }
    }
}

// TODO: mediump float may not be enough for GLES2 in iOS.
// see the following discussion: https://github.com/memononen/nanovg/issues/46
static VERT: &[u8] = b"
uniform vec2 viewSize;
attribute vec2 vertex;
attribute vec2 tcoord;
varying vec2 ftcoord;
varying vec2 fpos;
void main(void) {
    ftcoord = tcoord;
    fpos = vertex;
    gl_Position = vec4(2.0*vertex.x/viewSize.x - 1.0, 1.0 - 2.0*vertex.y/viewSize.y, 0, 1);
}
\0";

static FRAG: &[u8] = b"
#define UNIFORMARRAY_SIZE 11

//precision highp float;

uniform vec4 frag[UNIFORMARRAY_SIZE];
uniform sampler2D tex;
varying vec2 ftcoord;
varying vec2 fpos;

#define scissorMat mat3(frag[0].xyz, frag[1].xyz, frag[2].xyz)
#define paintMat mat3(frag[3].xyz, frag[4].xyz, frag[5].xyz)
#define innerCol frag[6]
#define outerCol frag[7]
#define scissorExt frag[8].xy
#define scissorScale frag[8].zw
#define extent frag[9].xy
#define radius frag[9].z
#define feather frag[9].w
#define strokeMult frag[10].x
#define strokeThr frag[10].y
#define texType int(frag[10].z)
#define type int(frag[10].w)

float sdroundrect(vec2 pt, vec2 ext, float rad) {
    vec2 ext2 = ext - vec2(rad,rad);
    vec2 d = abs(pt) - ext2;
    return min(max(d.x,d.y),0.0) + length(max(d,0.0)) - rad;
}

// Scissoring
float scissorMask(vec2 p) {
    vec2 sc = (abs((scissorMat * vec3(p,1.0)).xy) - scissorExt);
    sc = vec2(0.5,0.5) - sc * scissorScale;
    return clamp(sc.x,0.0,1.0) * clamp(sc.y,0.0,1.0);
}

// Stroke - from [0..1] to clipped pyramid, where the slope is 1px.
float strokeMask() {
    return min(1.0, (1.0-abs(ftcoord.x*2.0-1.0))*strokeMult) * min(1.0, ftcoord.y);
}

void main(void) {
    vec4 result;
    float scissor = scissorMask(fpos);

    float strokeAlpha = strokeMask();
    if (strokeAlpha < strokeThr) discard;

    if (type == 0) {            // Gradient
        // Calculate gradient color using box gradient
        vec2 pt = (paintMat * vec3(fpos,1.0)).xy;
        float d = clamp((sdroundrect(pt, extent, radius) + feather*0.5) / feather, 0.0, 1.0);
        vec4 color = mix(innerCol,outerCol,d);
        // Combine alpha
        color *= strokeAlpha * scissor;
        result = color;
    } else if (type == 1) {        // Image
        // Calculate color fron texture
        vec2 pt = (paintMat * vec3(fpos,1.0)).xy / extent;
        vec4 color = texture2D(tex, pt);
        if (texType == 1) color = vec4(color.xyz*color.w,color.w);
        if (texType == 2) color = vec4(color.x);
        // Apply color tint and alpha.
        color *= innerCol;
        // Combine alpha
        color *= strokeAlpha * scissor;
        result = color;
    } else if (type == 2) {        // Stencil fill
        result = vec4(1,1,1,1);
    } else if (type == 3) {        // Textured tris
        vec4 color = texture2D(tex, ftcoord);
        if (texType == 1) color = vec4(color.xyz*color.w,color.w);
        if (texType == 2) color = vec4(color.x);
        color *= scissor;
        result = color * innerCol;
    }
    gl_FragColor = result;
}\0";
