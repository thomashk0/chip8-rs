use std::ffi::{c_void, CStr};
use std::mem::size_of;

use gl::types::{GLfloat, GLsizei, GLsizeiptr, GLuint};

use crate::gl_scene::shaders::Shader;

mod shaders {
    use std::ffi::CString;

    use gl::types::{GLchar, GLenum, GLuint};

    unsafe fn compile_shader(src: &str, shader_type: GLenum) -> GLuint {
        let shader = gl::CreateShader(shader_type);
        let source = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &source.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);
        let mut status = i32::from(gl::TRUE);
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
        if status == (i32::from(gl::FALSE)) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len(len as usize);
            gl::GetShaderInfoLog(
                shader,
                len,
                std::ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );
            eprintln!("{}", String::from_utf8(buf).unwrap());
            unreachable!("shader compilation failed");
        }
        shader
    }

    unsafe fn link_shaders(vertex_shader: GLuint, fragment_shader: GLuint) -> GLuint {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
        gl::LinkProgram(program);
        let mut status = i32::from(gl::TRUE);
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
        if status == (i32::from(gl::FALSE)) {
            let mut len = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len(len as usize);
            gl::GetProgramInfoLog(
                program,
                len,
                std::ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );
            panic!(String::from_utf8(buf));
        }
        program
    }

    pub struct Shader {
        pub program: GLuint,
    }

    impl Drop for Shader {
        fn drop(&mut self) {
            unsafe {
                gl::DeleteProgram(self.program);
            }
        }
    }

    impl Shader {
        pub fn new(vertex_source: &str, fragment_source: &str) -> Shader {
            let program = unsafe {
                let vertex_shader = compile_shader(vertex_source, gl::VERTEX_SHADER);
                let fragment_shader = compile_shader(fragment_source, gl::FRAGMENT_SHADER);
                let program = link_shaders(vertex_shader, fragment_shader);
                gl::DeleteShader(vertex_shader);
                gl::DeleteShader(fragment_shader);

                program
            };
            Shader { program }
        }

        pub fn use_program(&self) {
            unsafe {
                gl::UseProgram(self.program);
            }
        }
    }
}

const VERTEX_SHADER_SRC: &str = r#"
    #version 330 core

    layout (location = 0) in vec3 aPos;
    layout (location = 1) in vec2 aTexCoord;

    out vec2 TexCoord;
    void main() {
       TexCoord = aTexCoord;
       gl_Position = vec4(aPos, 1.0);
    }
"#;

const FRAGMENT_SHADER_SRC: &str = r#"
    #version 330 core

    out vec4 FragColor;
    in vec2 TexCoord;

    uniform sampler2D screen;

    void main() {
       FragColor = texture(screen, TexCoord);
    }
"#;

// x   y    z    u    v
const SCREEN_QUAD_VERTICES: [f32; 20] = [
    1.0, 1.0, 0.0, 1.0, 1.0, /* */
    1.0, -1.0, 0.0, 1.0, 0.0, /* */
    -1.0, -1.0, 0.0, 0.0, 0.0, /* */
    -1.0, 1.0, 0.0, 0.0, 1.0,
];

const SCREEN_QUAD_IDX: [u32; 6] = [0, 1, 3, 1, 2, 3];

// TODO: implement Drop for vao, vbo, ebo, ...
pub struct Scene {
    shader: Shader,

    vao: GLuint,
    vbo: GLuint,
    ebo: GLuint,
    texture: GLuint,
}

impl Scene {
    pub fn new() -> Self {
        let (mut vao, mut vbo, mut ebo, mut texture) = (0, 0, 0, 0);
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::GenBuffers(1, &mut ebo);

            gl::BindVertexArray(vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (SCREEN_QUAD_VERTICES.len() * size_of::<GLfloat>()) as GLsizeiptr,
                SCREEN_QUAD_VERTICES.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (SCREEN_QUAD_IDX.len() * size_of::<GLfloat>()) as GLsizeiptr,
                SCREEN_QUAD_IDX.as_ptr() as *const c_void,
                gl::STATIC_DRAW,
            );

            let stride = 5 * size_of::<GLfloat>() as GLsizei;

            // position attribute
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, std::ptr::null());
            gl::EnableVertexAttribArray(0);

            // texture attribute
            gl::VertexAttribPointer(
                1,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (3 * size_of::<GLfloat>()) as *const c_void,
            );
            gl::EnableVertexAttribArray(1);

            // load and create texture
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        }
        Scene {
            shader: Shader::new(VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC),
            vao,
            vbo,
            ebo,
            texture,
        }
    }

    pub fn render(&self, framebuffer: &[u32], width: u32, height: u32) {
        assert_eq!(framebuffer.len(), (width * height) as usize);
        // Update texture data
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as i32,
                width as GLsizei,
                height as GLsizei,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                framebuffer.as_ptr() as *const c_void,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
            gl::ActiveTexture(gl::TEXTURE0);
            self.shader.use_program();
            let var_id = CStr::from_bytes_with_nul(b"screen\0").unwrap();
            let screen_uniform = gl::GetUniformLocation(self.shader.program, var_id.as_ptr());
            gl::Uniform1i(screen_uniform, 0);
            gl::BindVertexArray(self.vao);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
        }
    }
}


impl Drop for Scene {
    fn drop(&mut self) {
        let buffers = [self.vbo, self.ebo];
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao as *const GLuint);
            gl::DeleteBuffers(2, buffers.as_ptr());
            gl::DeleteTextures(1, &self.texture as *const GLuint);
        }
    }
}