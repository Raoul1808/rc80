use eframe::{
    egui::{self, mutex::Mutex},
    egui_glow, glow,
};
use std::mem::size_of;
use std::sync::Arc;

use rc80_core::System;

struct EmuApp {
    render: Arc<Mutex<EmuRender>>,
    sys: System,
}

impl EmuApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut sys = System::default();
        let bytes = include_bytes!("/home/mew/Downloads/1-chip8-logo.ch8");
        sys.load(bytes);
        let gl = cc.gl.as_ref().expect("glow backend is not enabled");
        Self {
            render: Arc::new(Mutex::new(EmuRender::new(gl))),
            sys,
        }
    }

    fn custom_painting(&mut self, ui: &mut egui::Ui) {
        let (rect, _) = ui.allocate_exact_size(
            egui::Vec2::new(640., 320.),
            egui::Sense::focusable_noninteractive(),
        );

        let render = self.render.clone();

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(egui_glow::CallbackFn::new(move |_info, painter| {
                render.lock().paint(painter.gl());
            })),
        };
        ui.painter().add(callback);
    }
}

impl eframe::App for EmuApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello eframe!");
            if ui.button("Step emulation").clicked() {
                self.sys.step();
            }
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                self.custom_painting(ui);
            });
        });
    }
}

struct EmuRender {
    program: glow::Program,
    vertex_array: glow::VertexArray,
}

impl EmuRender {
    fn new(gl: &glow::Context) -> Self {
        use glow::HasContext as _;

        unsafe {
            let program = gl.create_program().expect("Cannot create program");
            let shaders = [
                (glow::VERTEX_SHADER, include_str!("../res/shader.vert")),
                (glow::FRAGMENT_SHADER, include_str!("../res/shader.frag")),
            ];
            let shaders: Vec<_> = shaders
                .into_iter()
                .map(|(shader_type, shader_source)| {
                    let shader = gl.create_shader(shader_type).expect("cannot create shader");
                    gl.shader_source(shader, shader_source);
                    gl.compile_shader(shader);
                    assert!(
                        gl.get_shader_compile_status(shader),
                        "Failed to compile custom_3d_glow {shader_type}: {}",
                        gl.get_shader_info_log(shader)
                    );
                    gl.attach_shader(program, shader);
                    shader
                })
                .collect();
            gl.link_program(program);
            gl.use_program(Some(program));
            assert!(
                gl.get_program_link_status(program),
                "{}",
                gl.get_program_info_log(program)
            );
            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            let vertex_array = gl
                .create_vertex_array()
                .expect("cannot create vertex array");

            gl.bind_vertex_array(Some(vertex_array));

            let triangle_vertices = [5.0f32, 10.0f32, 0.0f32, 0.0f32, 10.0f32, 0.0f32];
            let triangle_vertices_u8: &[u8] = core::slice::from_raw_parts(
                triangle_vertices.as_ptr() as *const u8,
                triangle_vertices.len() * core::mem::size_of::<f32>(),
            );

            let buffer = gl.create_buffer().expect("cannot create buffer");
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, triangle_vertices_u8, glow::STATIC_DRAW);
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(
                0,
                2,
                glow::FLOAT,
                false,
                (2 * size_of::<f32>()) as i32,
                0,
            );

            use rc80_core::{SCREEN_HEIGHT, SCREEN_WIDTH};
            #[rustfmt::skip]
            let proj = [
                2. / SCREEN_WIDTH as f32, 0., 0., -1.,
                0., -2. / SCREEN_HEIGHT as f32, 0., 1.,
                0., 0., -1., 0.,
                0., 0., 0., 1.,
            ];
            let proj_uniform = gl
                .get_uniform_location(program, "u_projection")
                .expect("cannot find projection uniform location");
            gl.uniform_matrix_4_f32_slice(Some(&proj_uniform), true, &proj);

            Self {
                program,
                vertex_array,
            }
        }
    }

    fn paint(&self, gl: &glow::Context) {
        use glow::HasContext as _;
        unsafe {
            gl.use_program(Some(self.program));
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
            gl.bind_vertex_array(None);
            gl.use_program(None);
        }
    }
}

fn main() {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_min_inner_size((800., 640.)),
        ..Default::default()
    };
    eframe::run_native(
        "rc80 Desktop",
        native_options,
        Box::new(|cc| Box::new(EmuApp::new(cc))),
    )
    .unwrap()
}
