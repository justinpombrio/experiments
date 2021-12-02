#[macro_use]
extern crate glium;
extern crate image;

use std::cmp;
use std::default::Default;
use std::f32;

#[derive(Debug, Copy, Clone)]
struct Rectangle {
    center: [f32; 2],
    width: f32,
    height: f32,
}

impl Rectangle {
    fn vertices(self) -> [Vertex; 6] {
        let [x, y] = self.center;
        let [half_w, half_h] = [self.width / 2.0, self.height / 2.0];
        let (x_min, x_max) = (x - half_w, x + half_w);
        let (y_min, y_max) = (y - half_h, y + half_h);
        let v1 = Vertex {
            position: [x_min, y_max, 0.0],
            texture_coords: [0.0, 1.0],
        };
        let v2 = Vertex {
            position: [x_min, y_min, 0.0],
            texture_coords: [0.0, 0.0],
        };
        let v3 = Vertex {
            position: [x_max, y_max, 0.0],
            texture_coords: [1.0, 1.0],
        };
        let v4 = Vertex {
            position: [x_max, y_min, 0.0],
            texture_coords: [1.0, 0.0],
        };
        [v1, v2, v3, v2, v3, v4]
    }
}

#[derive(Debug, Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    texture_coords: [f32; 2],
}

#[derive(Debug, Copy, Clone)]
struct Light {
    position: [f32; 3],
    color: [f32; 4],
}

impl Default for Light {
    fn default() -> Light {
        Light {
            position: [0.0, 0.0, 0.0],
            color: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

fn load_texture(display: &glium::Display, path: &str) -> glium::texture::SrgbTexture2d {
    let image = image::open(path).unwrap().to_rgba();
    let image_dimensions = image.dimensions();
    let glium_image =
        glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    glium::texture::SrgbTexture2d::new(display, glium_image).unwrap()
}

fn load_texture_map(display: &glium::Display, path: &str) -> glium::texture::Texture2d {
    let image = image::open(path).unwrap().to_rgba();
    let image_dimensions = image.dimensions();
    let glium_image =
        glium::texture::RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
    glium::texture::Texture2d::new(display, glium_image).unwrap()
}

/*
The glium_text crate assumes a particular (and old) version of glium.

#[derive(Debug, Copy, Clone)]
pub struct Font(usize);

struct Fonts<'d> {
    display: &'d glium::Display,
    system: glium_text::TextSystem,
    fonts: Vec<glium_text::FontTexture>,
}

impl<'d> Fonts<'d> {
    fn new(display: &glium::Display) -> Fonts {
        Fonts {
            display,
            system: glium_text::TextSystem::new(display),
            fonts: vec![],
        }
    }

    fn load_font(&mut self, path: &str) -> Font {
        let file = File::open(path).expect("Failed to open font file");
        let font_texture = glium_text::FontTexture::new(&self.display, file, FONT_SIZE);
        let index = self.fonts.len();
        self.fonts.push(font_texture);
        Font(index)
    }
}
*/

fn main() {
    #[allow(unused_imports)]
    use glium::{glutin, Surface};

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    implement_vertex!(Vertex, position, texture_coords);

    let rectangle = Rectangle {
        center: [0.0, 0.2],
        width: 1.5,
        height: 1.5,
    };
    let vertices = rectangle.vertices();
    let shape = glium::vertex::VertexBuffer::new(&display, &vertices).unwrap();

    let diffuse_texture = load_texture(&display, "glium-example-assets/diffuse.png");
    let specular_map = load_texture_map(&display, "glium-example-assets/specular.png");
    let normal_map = load_texture_map(&display, "glium-example-assets/normal.png");

    let vertex_shader_src = r#"
        #version 150

        in vec3 position;
        in vec2 texture_coords;

        out vec3 v_normal;
        out vec3 v_position;
        out vec2 v_texture_coords;

        void main() {
            v_texture_coords = texture_coords;
            v_normal = vec3(0.0, 0.0, -1.0);
            gl_Position = vec4(position, 1.0);
            v_position = vec3(position);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        in vec3 v_normal;
        in vec3 v_position;
        in vec2 v_texture_coords;

        out vec4 color;

        uniform int num_lights;
        uniform vec3 pos_light0; uniform vec4 color_light0;
        uniform vec3 pos_light1; uniform vec4 color_light1;
        uniform vec3 pos_light2; uniform vec4 color_light2;
        uniform vec3 pos_light3; uniform vec4 color_light3;
        uniform vec3 pos_light4; uniform vec4 color_light4;
        uniform vec3 pos_light5; uniform vec4 color_light5;

        uniform sampler2D diffuse_texture;
        uniform sampler2D normal_texture;
        uniform sampler2D specular_texture;

        mat3 cotangent_frame(vec3 normal, vec3 pos, vec2 uv) {
            vec3 dp1 = dFdx(pos);
            vec3 dp2 = dFdy(pos);
            vec2 duv1 = dFdx(uv);
            vec2 duv2 = dFdy(uv);

            vec3 dp2perp = cross(dp2, normal);
            vec3 dp1perp = cross(normal, dp1);
            vec3 T = dp2perp * duv1.x + dp1perp * duv2.x;
            vec3 B = dp2perp * duv1.y + dp1perp * duv2.y;

            float invmax = inversesqrt(max(dot(T, T), dot(B, B)));
            return mat3(T * invmax, B * invmax, normal);
        }

        void main() {
            int i;

            vec3 pos_lights[6] = vec3[6](
                pos_light0, pos_light1, pos_light2, pos_light3, pos_light4, pos_light5
            );
            vec4 color_lights[6] = vec4[6](
                color_light0, color_light1, color_light2, color_light3, color_light4, color_light5
            );

            float alpha = texture(diffuse_texture, v_texture_coords).a;
            vec3 diffuse_color = texture(diffuse_texture, v_texture_coords).rgb;

            vec3 normal_map = texture(normal_texture, v_texture_coords).rgb;
            mat3 tbn = cotangent_frame(v_normal, v_position, v_texture_coords);
            vec3 real_normal = normalize(tbn * -(2.0 * normal_map - 1.0));

            float specular_map = texture(specular_texture, v_texture_coords).r;

            vec3 ambient_lighting = diffuse_color * 0.1;
            vec3 diffuse_lighting = vec3(0.0, 0.0, 0.0);
            vec3 specular_lighting = vec3(0.0, 0.0, 0.0);

            vec3 camera_dir = normalize(-v_position);
            for (i = 0; i < num_lights; i++) {
                vec3 light_pos = pos_lights[i];
                vec3 light_color = color_lights[i].rgb * color_lights[i].a;

                float diffuse_brightness = max(dot(real_normal, normalize(light_pos)), 0.0);
                diffuse_brightness /= distance(v_position, light_pos) * distance(v_position, light_pos);
                diffuse_lighting += diffuse_brightness * diffuse_color * light_color;
                
                vec3 half_direction = normalize(normalize(light_pos) + camera_dir);
                float specular_brightness = pow(max(dot(half_direction, real_normal), 0.0), 16.0) * specular_map;
                specular_lighting += specular_brightness * light_color;
            }

            color = vec4(ambient_lighting + diffuse_lighting + specular_lighting, alpha);
        }
    "#;

    let program =
        glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None)
            .unwrap();

    let mut t = 0.0;

    event_loop.run(move |event, _, control_flow| {
        t += 0.001;
        if t >= 1.0 {
            t = 0.0;
        }
        let next_frame_time =
            std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                _ => return,
            },
            glutin::event::Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glutin::event::StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);

        let theta = 2.0 * 3.1416 * t;
        let light1 = Light {
            position: [f32::cos(theta), f32::sin(theta), 0.1],
            color: [1.0, 0.0, 0.0, 0.5],
        };
        let light2 = Light {
            position: [0.0, 0.0, 0.5],
            color: [1.0, 1.0, 1.0, 0.5],
        };
        let lights = vec![light2, light1];

        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            blend: glium::Blend::alpha_blending(),
            ..Default::default()
        };

        let pos_light =
            |i: usize| -> [f32; 3] { lights.get(i).copied().unwrap_or_default().position };
        let color_light =
            |i: usize| -> [f32; 4] { lights.get(i).copied().unwrap_or_default().color };
        target
            .draw(
                &shape,
                glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
                &program,
                &uniform! {
                    num_lights: cmp::max(lights.len(), 6) as i32,
                    pos_light0: pos_light(0), color_light0: color_light(0),
                    pos_light1: pos_light(1), color_light1: color_light(1),
                    pos_light2: pos_light(2), color_light2: color_light(2),
                    pos_light3: pos_light(3), color_light3: color_light(3),
                    pos_light4: pos_light(4), color_light4: color_light(4),
                    pos_light5: pos_light(5), color_light5: color_light(5),
                    diffuse_texture: &diffuse_texture,
                    normal_texture: &normal_map,
                    specular_texture: &specular_map,
                },
                &params,
            )
            .unwrap();
        target.finish().unwrap();
    });
}
