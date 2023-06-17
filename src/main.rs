use miniquad::conf::Conf;
use miniquad::{
    Bindings, Buffer, BufferLayout, BufferType, Context, EventHandler, MouseButton, Pipeline,
    Shader, ShaderMeta, TouchPhase, UniformBlockLayout, UniformDesc, UniformType, VertexAttribute,
    VertexFormat, KeyMods, KeyCode
};

use quad_rand as qrand;
use std::time::Instant;


#[repr(C)]
struct Vec2 {
    x: f32,
    y: f32,
}
#[repr(C)]
struct Vertex {
    pos: Vec2,
}
#[repr(C)]
struct Uniforms {
    transform: [f32; 16],
    cxmin: f32,
    cxmax: f32,
    cymin: f32,
    cymax: f32,
    color_seed: f32,
}

#[derive(Copy, Clone, Debug)]
enum Action {
    Idle,
    ZoomingIn(f32, f32),
    ZoomingOut(f32, f32),
}

struct Lyapunov {
    pipeline: Pipeline,
    bindings: Bindings,
    zoom: f32,
    center: (f32, f32),
    action: Action,
    color_seed: f32,
    cxmin: f32,
    cxmax: f32,
    cymin: f32,
    cymax: f32,
}

// Generate a random float in a given range
fn random_f32_range(min: f32, max: f32) -> f32 {
    let range = max - min;
    let random = qrand::gen_range(0.0, 1.0);
    min + range * random
}

impl Lyapunov {
    // Create a new Lyapunov fractal with randomly generated parameters
    fn new(ctx: &mut Context) -> Self {
        // Define vertices for the square on which the fractal will be drawn
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
        let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);

        // Indices for the square
        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
        let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);

        // Bindings for the pipeline
        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer: index_buffer,
            images: Vec::new(),
        };

        // Create the shader
        let shader = Shader::new(ctx, SHADER_VERTEX, SHADER_FRAGMENT, meta());

        // Create the pipeline
        let pipeline = Pipeline::new(
            ctx,
            &[BufferLayout::default()],
            &[VertexAttribute::new("pos", VertexFormat::Float2)],
            shader.unwrap(),
        );

        let now = Instant::now();
        let seed = now.elapsed().as_nanos() as u64;
        
        qrand::srand(seed);

        // Color range (sine)
        let color_seed_min = 0.0;
        let color_seed_max = 10.0;
        let color_seed = random_f32_range(color_seed_min, color_seed_max);

        // Logistic map parameters
        let cxmin = random_f32_range(1.5, 3.0);
        let cxmax = random_f32_range(cxmin, 4.0);
        let cymin = random_f32_range(1.5, 3.0);
        let cymax = random_f32_range(cymin, 4.0);

        Lyapunov {
            pipeline,
            bindings,
            zoom: 1.0,
            center: (0.0, 0.0),
            action: Action::Idle,
            color_seed,
            cxmin,
            cxmax,
            cymin,
            cymax,
        }
    }

    // Returns two floats (x and y) from -0.5 to 0.5, with (0.0, 0.0) being the center of the screen
    fn norm_mouse_pos(self: &Self, ctx: &mut Context, x: f32, y: f32) -> (f32, f32) {
        let screen_size = ctx.screen_size();
        let pos = (
            4.0 * (x / screen_size.0 - 0.5).powi(3),
            4.0 * (y / screen_size.1 - 0.5).powi(3),
        );

        pos
    }

    fn update_parameters(&mut self) {
        self.color_seed = random_f32_range(0.0, 10.0);
        self.cxmin = random_f32_range(1.5, 3.0);
        self.cxmax = random_f32_range(self.cxmin, 4.0);
        self.cymin = random_f32_range(1.5, 3.0);
        self.cymax = random_f32_range(self.cymin, 4.0);
    }
}

impl EventHandler for Lyapunov {
    fn update(&mut self, _ctx: &mut Context) {
        // zoom in/out
        match self.action {
            Action::ZoomingIn(x, y) => {
                self.zoom *= 1.01;
                self.center.0 -= x / self.zoom;
                self.center.1 += y / self.zoom;
            }
            Action::ZoomingOut(x, y) => {
                self.zoom /= 1.01;
                self.center.0 += x / self.zoom;
                self.center.1 -= y / self.zoom;
            }
            _ => {}
        }
    }

    fn draw(&mut self, ctx: &mut Context) {
        // Draw the Lyapunov set
        ctx.begin_default_pass(Default::default());

        ctx.apply_pipeline(&self.pipeline);
        ctx.apply_bindings(&self.bindings);

        // Make sure to not stretch
        let screen_size = ctx.screen_size();
        let ratio = screen_size.1 / screen_size.0;
        let (mut scale_x, mut scale_y) = if ratio <= 1.0 {
            (ratio, 1.0)
        } else {
            (1.0, 1.0 / ratio)
        };

        scale_x *= self.zoom;
        scale_y *= self.zoom;

        #[rustfmt::skip]
        ctx.apply_uniforms(&Uniforms {
            transform: [
                scale_x, 0.0, 0.0, 0.0,
                0.0, scale_y, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                (scale_x * self.center.0), (scale_y * self.center.1), 0.0, 1.0,
            ],
            cxmin: self.cxmin,
            cxmax: self.cxmax,
            cymin: self.cymin,
            cymax: self.cymax,
            color_seed: self.color_seed,
        });

        ctx.draw(0, 2 * 3, 1);

        ctx.end_render_pass();

        ctx.commit_frame();
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        let pos = self.norm_mouse_pos(ctx, x, y);

        if let MouseButton::Left = button {
            self.action = Action::ZoomingIn(pos.0, pos.1);
        } else if let MouseButton::Right = button {
            self.action = Action::ZoomingOut(pos.0, pos.1);
        }
    }

    // Update parameters when pressing spacebar
    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods, _repeat: bool) {
        if keycode == KeyCode::Space {
            self.update_parameters();
        }
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, _b: MouseButton, _x: f32, _y: f32) {
        self.action = Action::Idle;
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32) {
        let pos = self.norm_mouse_pos(ctx, x, y);

        match self.action {
            Action::ZoomingIn(..) => {
                self.action = Action::ZoomingIn(pos.0, pos.1);
            }
            Action::ZoomingOut(..) => {
                self.action = Action::ZoomingOut(pos.0, pos.1);
            }
            _ => {}
        }
    }

    fn touch_event(&mut self, ctx: &mut Context, phase: TouchPhase, _id: u64, x: f32, y: f32) {
        let pos = self.norm_mouse_pos(ctx, x, y);

        match phase {
            TouchPhase::Started => {
                self.action = Action::ZoomingIn(pos.0, pos.1);
            }
            TouchPhase::Moved => {
                self.action = Action::ZoomingIn(pos.0, pos.1);
            }
            _ => {
                self.action = Action::Idle;
            }
        }
    }

    
}

fn main() {
    miniquad::start(Conf::default(), |mut ctx| Box::new(Lyapunov::new(&mut ctx)));
}

const SHADER_VERTEX: &str = r#"#version 100

uniform highp mat4 transform;

attribute highp vec2 pos;
varying highp vec2 texcoord;

void main() {
    gl_Position = transform * vec4(pos, 0, 1);
    texcoord = vec2(pos.x/2.0 + 0.5, 1.0 - (pos.y/2.0 + 0.5));
}"#;

// Computes the Lyapunov exponent for each pixel and assigns a color based on its value
const SHADER_FRAGMENT: &str = r#"#version 100

precision highp float;

uniform highp float cxmin;
uniform highp float cxmax;
uniform highp float cymin;
uniform highp float cymax;
uniform highp float color_seed;

varying highp vec2 texcoord;

const int max_iterations = 1000;
const int warm_up_iterations = 100;

// Generate a color based on the input value `t` and a seed value for variation
vec4 color_palette(float t) {
    float r = 0.5 + 0.5 * sin(3.0 * t + color_seed);
    float g = 0.5 + 0.5 * sin(3.0 * t + color_seed + 2.0);
    float b = 0.5 + 0.5 * sin(3.0 * t + color_seed + 4.0);
    return vec4(r, g, b, 1.0);
}

void main() {
    float scale_x = cxmax - cxmin;
    float scale_y = cymax - cymin;

    float a = texcoord.x * scale_x + cxmin;
    float b = texcoord.y * scale_y + cymin;

    float x = 0.5;
    float lyapunov_exponent = 0.0;

    // Warm up iterations to initialize x
    for (int i = 0; i < warm_up_iterations; i++) {
        x = fract(0.5 + ((mod(float(i), 2.0) == 0.0) ? a * x * (1.0 - x) : b * x * (1.0 - x)));
    }

    // Compute the Lyapunov exponent for the current pixel
    for (int i = 0; i < max_iterations; i++) {
        float rx = (mod(float(i), 2.0) == 0.0) ? a * x * (1.0 - x) : b * x * (1.0 - x);
        x = fract(0.5 + rx);
        lyapunov_exponent += log(abs((mod(float(i), 2.0) == 0.0 ? a : b) * (1.0 - 2.0 * x)));
    }

    lyapunov_exponent /= float(max_iterations);

    float color_value = (lyapunov_exponent + 2.0) / 4.0;
    gl_FragColor = color_palette(color_value);
}"#;

pub fn meta() -> ShaderMeta {
    ShaderMeta {
        images: vec![],
        uniforms: UniformBlockLayout {
            uniforms: vec![
                UniformDesc::new("transform", UniformType::Mat4),
                UniformDesc::new("cxmin", UniformType::Float1),
                UniformDesc::new("cxmax", UniformType::Float1),
                UniformDesc::new("cymin", UniformType::Float1),
                UniformDesc::new("cymax", UniformType::Float1),
                UniformDesc::new("color_seed", UniformType::Float1),
            ],
        },
    }
}
