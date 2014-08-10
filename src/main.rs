#![feature(phase)]

#[phase(plugin)]
extern crate gfx_macros;
extern crate getopts;
extern crate gfx;
extern crate glfw;
extern crate glfw_platform;
extern crate native;

use glfw_platform::BuilderExtension;

#[vertex_format]
struct Vertex {
    pos: [f32, ..2],
    color: [f32, ..3],
}

static VERTEX_SRC: gfx::ShaderSource = shaders! {
GLSL_120: b"
    #version 120
    attribute vec2 pos;
    attribute vec3 color;
    varying vec4 v_Color;
    void main() {
        v_Color = vec4(color, 1.0);
        gl_Position = vec4(pos, 0.0, 1.0);
    }
"
GLSL_150: b"
    #version 150 core
    in vec2 pos;
    in vec3 color;
    out vec4 v_Color;
    void main() {
        v_Color = vec4(color, 1.0);
        gl_Position = vec4(pos, 0.0, 1.0);
    }
"
};

static FRAGMENT_SRC: gfx::ShaderSource = shaders! {
GLSL_120: b"
    #version 120
    varying vec4 v_Color;
    void main() {
        gl_FragColor = v_Color;
    }
"
GLSL_150: b"
    #version 150 core
    in vec4 v_Color;
    out vec4 o_Color;
    void main() {
        o_Color = v_Color;
    }
"
};

// We need to run on the main thread for GLFW, so ensure we are using the `native` runtime. This is
// technically not needed, since this is the default, but it's not guaranteed.
#[start]
fn start(argc: int, argv: *const *const u8) -> int {
     native::start(argc, argv, main)
}

fn main() {
    // checking for the --gl-init flag
    let use_glinit = {
        use std::os;
        use getopts::{optflag, getopts};

        let args: Vec<String> = os::args();

        let options = [
            optflag("i", "gl-init", "use gl-init instead of glfw")
        ];

        let matches = match getopts(args.tail(), options) {
            Ok(m) => { m }
            Err(f) => {
                println!("{}", f.to_string());
                println!("Usage: {} [options]", args[0]);
                println!("--gl-init\t\tUse gl-init instead of glfw");
                return
            }
        };

        matches.opt_present("i")
    };

    if use_glinit {
    } else {
        let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

        let (mut window, events) = glfw_platform::WindowBuilder::new(&glfw)
            .title("[GLFW] Triangle example #gfx-rs!")
            .try_modern_context_hints()
            .create()
            .expect("Failed to create GLFW window.");

        glfw.set_error_callback(glfw::FAIL_ON_ERRORS);
        window.set_key_polling(true); // so we can quit when Esc is pressed
        let (w, h) = window.get_size();

        let mut device = gfx::build()
            .with_glfw_window(&mut window)
            .with_queue_size(1)
            .spawn(proc(r) render(r, w as u16, h as u16))
            .unwrap();

        'main: loop {
            glfw.poll_events();
            if window.should_close() {
                break 'main;
            }
            // quit when Esc is pressed.
            for (_, event) in glfw::flush_messages(&events) {
                match event {
                    glfw::KeyEvent(glfw::KeyEscape, _, glfw::Press, _) => break 'main,
                    _ => {},
                }
            }
            device.update();
        }
    }
}

fn render(mut renderer: gfx::Renderer, width: u16, height: u16) {
    let frame = gfx::Frame::new(width as u16, height as u16);
    let state = gfx::DrawState::new();

    let vertex_data = vec![
        Vertex { pos: [ -0.5, -0.5 ], color: [1.0, 0.0, 0.0] },
        Vertex { pos: [ 0.5, -0.5 ], color: [0.0, 1.0, 0.0]  },
        Vertex { pos: [ 0.0, 0.5 ], color: [0.0, 0.0, 1.0]  }
    ];

    let mesh = renderer.create_mesh(vertex_data);
    let program = renderer.create_program(VERTEX_SRC.clone(), FRAGMENT_SRC.clone());

    let clear = gfx::ClearData {
        color: Some(gfx::Color([0.3, 0.3, 0.3, 1.0])),
        depth: None,
        stencil: None,
    };

    while !renderer.should_finish() {
        renderer.clear(clear, frame);
        renderer.draw(&mesh, mesh.get_slice(), &frame, &program, &state)
            .unwrap();
        renderer.end_frame();
        for err in renderer.errors() {
            println!("Renderer error: {}", err);
        }
    }
}
