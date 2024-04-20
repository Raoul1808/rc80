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
        let bytes = include_bytes!("/home/mew/Downloads/2-ibm-logo.ch8");
        sys.load(bytes);
        let gl = cc.gl.as_ref().expect("glow backend is not enabled");
        sys.pixels[0] = 1;
        sys.pixels[1] = 1;
        sys.pixels[727] = 1;
        sys.pixels[1116] = 1;
        println!("{:b}", sys.pixels[0]);
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
        let pixels = self.sys.pixels;

        let callback = egui::PaintCallback {
            rect,
            callback: Arc::new(egui_glow::CallbackFn::new(move |_info, painter| {
                render.lock().update_buffers(pixels, painter.gl());
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
    vertex_buffer: glow::Buffer,
    index_buffer: glow::Buffer,
    index_count: usize,
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

            let vertex_buffer = gl.create_buffer().expect("cannot create buffer");
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &[0], glow::DYNAMIC_DRAW);
            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(
                0,
                2,
                glow::FLOAT,
                false,
                (2 * size_of::<f32>()) as i32,
                0,
            );

            let index_buffer = gl.create_buffer().expect("cannot create buffer");
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(index_buffer));
            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, &[0], glow::STREAM_DRAW);

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
                vertex_buffer,
                index_buffer,
                index_count: 0,
            }
        }
    }

    fn update_buffers(&mut self, pixels: rc80_core::ScreenPixels, gl: &glow::Context) {
        use glow::HasContext as _;
        unsafe {
            let mut vertices = Vec::<f32>::new();
            let mut indices = Vec::<u32>::new();
            let mut next_index = 0;
            for (index, pixel) in pixels.into_iter().enumerate() {
                use rc80_core::SCREEN_WIDTH;
                if pixel == 1 {
                    let x = (index % SCREEN_WIDTH) as f32;
                    let y = (index / SCREEN_WIDTH) as f32;
                    vertices.extend_from_slice(&[x, y, x + 1., y, x + 1., y + 1., x, y + 1.]);
                    let i = next_index;
                    indices.extend_from_slice(&[i, i + 1, i + 2, i + 2, i + 3, i]);
                    next_index += 4;
                }
            }
            let vertices_u8: &[u8] = core::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                vertices.len() * core::mem::size_of::<f32>(),
            );
            let indices_u8: &[u8] = core::slice::from_raw_parts(
                indices.as_ptr() as *const u8,
                indices.len() * core::mem::size_of::<u32>(),
            );
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vertex_buffer));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices_u8, glow::STREAM_DRAW);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.index_buffer));
            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, indices_u8, glow::STREAM_DRAW);
            self.index_count = indices.len();
        }
    }

    fn paint(&self, gl: &glow::Context) {
        use glow::HasContext as _;
        unsafe {
            gl.use_program(Some(self.program));
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.draw_elements(
                glow::TRIANGLES,
                self.index_count as i32,
                glow::UNSIGNED_INT,
                0,
            );
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
