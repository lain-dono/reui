use super::gl::{
    self,
    types::{GLint, GLuint},
};
use std::ptr::null;

pub struct Shader {
    prog: GLuint,
    loc_viewsize: GLint,
    loc_frag: GLint,
}

impl Shader {
    pub fn new() -> Self {
        let (vshader, fshader) = (VERT.as_ptr() as *const i8, FRAG.as_ptr() as *const i8);

        unsafe {
            let prog = gl::CreateProgram();
            let vert = gl::CreateShader(gl::VERTEX_SHADER);
            let frag = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(vert, 1, &vshader, null());
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

            gl::BindAttribLocation(prog, 0, b"a_Position\0".as_ptr() as *const i8);
            gl::BindAttribLocation(prog, 1, b"a_TexCoord\0".as_ptr() as *const i8);

            gl::LinkProgram(prog);
            /*
            gl::GetProgramiv(prog, GL_LINK_STATUS, &status);
            assert!(status == 1);
            */

            Self {
                prog,
                loc_viewsize: gl::GetUniformLocation(prog, b"viewSize\0".as_ptr() as *const i8),
                loc_frag: gl::GetUniformLocation(prog, b"frag\0".as_ptr() as *const i8),
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

    pub fn bind_frag(&self, array: &[f32; 7 * 4 + 1]) {
        unsafe {
            gl::Uniform4fv(self.loc_frag, 7, &(array[0]));
        }
    }

    pub fn bind_view(&self, view: *const [f32; 2]) {
        unsafe {
            //gl::Uniform1i(self.loc_tex, 0);
            gl::Uniform2fv(self.loc_viewsize, 1, view as *const f32);
        }
    }
}

// TODO: mediump float may not be enough for GLES2 in iOS.
// see the following discussion: https://github.com/memononen/nanovg/issues/46
static VERT: &[u8] = b"
uniform vec2 viewSize;

attribute vec2 a_Position;
attribute vec2 a_TexCoord;

varying vec2 v_Position;
varying vec2 v_TexCoord;

void main() {
    v_TexCoord = a_TexCoord;
    v_Position = a_Position;
    gl_Position = vec4(
        2.0 * a_Position.x / viewSize.x - 1.0,
        1.0 - 2.0 * a_Position.y / viewSize.y,
        0.0, 1.0);
}
\0";

static FRAG: &[u8] = b"
#define UNIFORMARRAY_SIZE 7

//precision highp float;

varying vec2 v_Position;
varying vec2 v_TexCoord;

uniform vec4 frag[UNIFORMARRAY_SIZE];

#define scissorTransform frag[0]
#define paintTransform frag[1]

#define innerCol frag[2]
#define outerCol frag[3]
#define scissorExt frag[4].xy
#define scissorScale frag[4].zw
#define extent frag[5].xy
#define radius frag[5].z
#define feather frag[5].w
#define strokeMult frag[6].x
#define strokeThr frag[6].y

#define type int(frag[6].w)

float sdroundrect(vec2 pt, vec2 ext, float rad) {
    vec2 ext2 = ext - vec2(rad,rad);
    vec2 d = abs(pt) - ext2;
    return min(max(d.x,d.y),0.0) + length(max(d,0.0)) - rad;
}

vec2 applyTransform(vec4 transform, vec2 pt) {
    float re = transform.x;
    float im = transform.y;
    return transform.zw + vec2(pt.x * re - pt.y * im, pt.x * im + pt.y * re);
}

// Scissoring
float scissorMask(vec2 p) {
    vec2 sc = vec2(0.5,0.5) -
        (abs(applyTransform(scissorTransform, p)) - scissorExt) * scissorScale;
    return clamp(sc.x,0.0,1.0) * clamp(sc.y,0.0,1.0);
}

// Stroke - from [0..1] to clipped pyramid, where the slope is 1px.
float strokeMask() {
    return min(1.0, (1.0-abs(v_TexCoord.x*2.0-1.0))*strokeMult) * min(1.0, v_TexCoord.y);
}

void main(void) {
    float scissor = scissorMask(v_Position);

    float strokeAlpha = strokeMask();
    if (strokeAlpha < strokeThr) {
        discard;
    }

    vec4 result;
    if (type == 0) {            // Gradient
        // Calculate gradient color using box gradient
        vec2 pt = applyTransform(paintTransform, v_Position);
        float d = clamp((sdroundrect(pt, extent, radius) + feather*0.5) / feather, 0.0, 1.0);
        vec4 color = mix(innerCol,outerCol,d);
        // Combine alpha
        color *= strokeAlpha * scissor;
        result = color;
    } else if (type == 2) {        // Stencil fill
        result = vec4(1,1,1,1);
    }

    gl_FragColor = result;
}\0";
