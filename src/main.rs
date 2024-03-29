use miniquad::*;

#[repr(C)]
struct Vec2 {
    x: f32,
    y: f32,
}

#[repr(C)]
struct Vertex {
    pos: Vec2,
}

struct Stage {
    ctx: Box<dyn RenderingBackend>,
    pipeline: Pipeline,
    bindings: Bindings,
    resolution: (f32, f32),
    center: (f32, f32),
    zoom: f32,
    iterations: i32,
}

impl Stage {
    pub fn new() -> Stage {
        let mut ctx: Box<dyn RenderingBackend> = window::new_rendering_backend();

        #[rustfmt::skip]
        let vertices: [Vertex; 4] = [
            Vertex {
                pos: Vec2 { x: -1.0, y: -1.0 },
            },
            Vertex {
                pos: Vec2 { x: 1.0, y: -1.0 },
            },
            Vertex {
                pos: Vec2 { x: 1.0, y: 1.0 },
            },
            Vertex {
                pos: Vec2 { x: -1.0, y: 1.0 },
            },
        ];
        let vertex_buffer = ctx.new_buffer(
            BufferType::VertexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&vertices),
        );

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = ctx.new_buffer(
            BufferType::IndexBuffer,
            BufferUsage::Immutable,
            BufferSource::slice(&indices),
        );

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer: index_buffer,
            images: vec![],
        };

        let shader = ctx
            .new_shader(
                ShaderSource::Glsl {
                    vertex: shader::VERTEX,
                    fragment: shader::FRAGMENT,
                },
                shader::meta(),
            )
            .unwrap();

        let pipeline = ctx.new_pipeline(
            &[BufferLayout::default()],
            &[VertexAttribute::new("in_pos", VertexFormat::Float2)],
            shader,
            PipelineParams::default(),
        );

        Stage {
            pipeline,
            bindings,
            ctx,
            resolution: (800.0, 600.0),
            center: (0.0, 0.0),
            zoom: 1.0,
            iterations: 100,
        }
    }
}

impl EventHandler for Stage {
    fn update(&mut self) {}

    fn draw(&mut self) {
        self.ctx.begin_default_pass(Default::default());

        self.ctx.apply_pipeline(&self.pipeline);
        self.ctx.apply_bindings(&self.bindings);
        self.ctx.apply_uniforms(UniformsSource::table(&shader::Uniforms {
            resolution: self.resolution,
            center: self.center,
            zoom: self.zoom,
            iterations: self.iterations,
        }));
        self.ctx.draw(0, 6, 1);

        self.ctx.end_render_pass();

        self.ctx.commit_frame();
    }
}

fn main() {
    let mut conf = conf::Conf::default();
    miniquad::start(conf, move || Box::new(Stage::new()));
}

mod shader {
    use miniquad::*;

    pub const VERTEX: &str = r#"
    #version 100
    attribute vec2 in_pos;
    attribute vec2 in_uv;

    varying lowp vec2 texcoord;

    void main() {
        gl_Position = vec4(in_pos, 0.0, 1.0);
        texcoord = in_uv;
    }
    "#;

    pub const FRAGMENT: &str = r#"#version 100
    precision mediump float;

    varying lowp vec2 texcoord;

    uniform vec2 resolution;
    uniform vec2 center;
    uniform float zoom;
    uniform int iterations;

    void main() {
        vec2 uv = (texcoord * 2.0 - 1.0) * vec2(resolution.x / resolution.y, 1.0);
        uv *= zoom;
        uv += center;

        vec2 c = uv;
        vec2 z = vec2(0.0);

        float iter = 0.0;
        for (int i = 0; i < iterations; i++) {
            z = vec2(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + c;
            if (dot(z, z) > 4.0) {
                break;
            }
            iter += 1.0;
        }

        float color = iter / float(iterations);
        gl_FragColor = vec4(color, color, color, 1.0);
    }
    "#;

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec![],
            uniforms: UniformBlockLayout {
                uniforms: vec![
                    UniformDesc::new("resolution", UniformType::Float2),
                    UniformDesc::new("center", UniformType::Float2),
                    UniformDesc::new("zoom", UniformType::Float1),
                    UniformDesc::new("iterations", UniformType::Int1),
                ],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub resolution: (f32, f32),
        pub center: (f32, f32),
        pub zoom: f32,
        pub iterations: i32,
    }
}