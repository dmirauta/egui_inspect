use egui_inspect::{
    eframe::glow::{self, HasContext, Program, VertexArray},
    logging::log::warn,
};
use std::sync::Arc;

/// print on gl error
#[macro_export]
macro_rules! pogle {
    ($gl: ident, $exp: expr) => {{
        use egui_inspect::logging::log::error;
        let local = $exp;
        match $gl.get_error() {
            0 => {}
            glow::INVALID_ENUM => error!("{} Encountered: Invalid enum", stringify!($exp)),
            glow::INVALID_VALUE => error!("{} Encountered: Invalid value", stringify!($exp)),
            glow::INVALID_OPERATION => {
                error!("{} Encountered: Invalid operation", stringify!($exp))
            }
            e => error!("{} Encountered: Error code {e}", stringify!($exp)),
        }
        local
    }};
}

/// a quad which covers the whole viewport
pub struct ViewportQuad {
    pub va: VertexArray,
    pub prog: Option<Program>,
}

static LARGE_TRI_VERT_SHADER: &str = include_str!("../viewport_tri_vertex.glsl");

impl ViewportQuad {
    pub fn new(gl: &Arc<glow::Context>, fragment_shader_source: &str) -> Self {
        unsafe {
            let va = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");

            let mut new = Self { va, prog: None };
            if let Err(e) = new.set_frag_shader(gl, fragment_shader_source) {
                warn!("Could not set frag shader: {e}")
            }
            new
        }
    }
    // NOTE: adapted from https://github.com/grovesNL/glow/blob/main/examples/hello/src/main.rs
    pub fn set_frag_shader(
        &mut self,
        gl: &Arc<glow::Context>,
        fragment_shader_source: &str,
    ) -> Result<(), String> {
        unsafe {
            gl.bind_vertex_array(Some(self.va));

            let program = gl.create_program().expect("Cannot create program");

            let mut shaders = Vec::with_capacity(2);

            let shader_version = if cfg!(target_arch = "wasm32") {
                "#version 300 es"
            } else {
                "#version 330"
            };

            for (shader_type, shader_source) in [
                (glow::VERTEX_SHADER, LARGE_TRI_VERT_SHADER),
                (glow::FRAGMENT_SHADER, fragment_shader_source),
            ] {
                let shader = gl.create_shader(shader_type).expect("Cannot create shader");
                gl.shader_source(shader, &format!("{}\n{}", shader_version, shader_source));
                gl.compile_shader(shader);
                if !gl.get_shader_compile_status(shader) {
                    return Err(gl.get_shader_info_log(shader));
                }
                gl.attach_shader(program, shader);
                shaders.push(shader);
            }

            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                return Err(gl.get_program_info_log(program));
            }

            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            gl.use_program(Some(program));

            if let Some(old_prog) = self.prog {
                gl.delete_program(old_prog);
            }
            self.prog = Some(program);
        }
        Ok(())
    }
}
