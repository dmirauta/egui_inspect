use egui_inspect::{
    eframe::{
        egui_glow,
        glow::{self, HasContext},
        CreationContext,
    },
    egui::{self, vec2, LayerId, Sense, Shape},
    logging::default_mixed_logger,
    EframeMain, EguiInspect,
};
use std::sync::Arc;
use viewport_quad::ViewportQuad;

mod viewport_quad;

static FRAG_SHADER: &str = include_str!("../test_fragment.glsl");
const ASPECT: f32 = 9.0 / 16.0;

#[derive(EframeMain)]
#[eframe_main(init = "FragViewport::init(_cc)")]
struct FragViewport {
    quad: ViewportQuad,
    t: f32,
}

impl FragViewport {
    fn init(cc: &CreationContext) -> Self {
        default_mixed_logger::<Self>();
        let gl = cc.gl.as_ref().unwrap().clone();
        Self {
            quad: ViewportQuad::new(&gl, FRAG_SHADER),
            t: 0.0,
        }
    }
    // TODO: make into widget?
    fn paint_viewport(&self, ui: &mut egui::Ui) {
        let available = ui.available_size();
        let size = match available.y / ASPECT < available.x {
            true => vec2(available.y / ASPECT, available.y),
            false => vec2(available.x, available.x * ASPECT),
        };
        let (rect, _) = ui.allocate_exact_size(size, Sense::empty());

        let t = self.t;
        let prog = self.quad.prog;
        let va = Some(self.quad.va);
        ui.ctx()
            .layer_painter(LayerId::background())
            .add(Shape::Callback(egui::PaintCallback {
                rect,
                callback: Arc::new(egui_glow::CallbackFn::new(move |_, painter| {
                    let gl = painter.gl();
                    unsafe {
                        pogle!(gl, gl.use_program(prog));
                        pogle!(gl, gl.bind_vertex_array(va));

                        if let Some(prog) = prog {
                            let loc = pogle!(gl, gl.get_uniform_location(prog, "t"));
                            pogle!(gl, gl.uniform_1_f32(loc.as_ref(), t));
                        }

                        pogle!(gl, gl.draw_arrays(glow::TRIANGLES, 0, 3));
                    }
                })),
            }));
    }
}

impl EguiInspect for FragViewport {
    fn inspect_mut(&mut self, _: &str, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("uniform:");
            ui.add(egui::Slider::new(&mut self.t, 0.0..=1.0));
        });
        self.paint_viewport(ui);
        ui.label("a widget directly after the viewport...");
    }
}
