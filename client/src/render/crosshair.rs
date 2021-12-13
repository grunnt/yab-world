use crate::render::mesh::{Mesh, Vertex};
use failure;
use gamework::video::*;
use gamework::*;
use gl;
use nalgebra_glm::*;
pub struct Crosshair {
    gl: gl::Gl,
    program: Program,
    model_uniform: Option<Uniform>,
    view_uniform: Option<Uniform>,
    projection_uniform: Option<Uniform>,
    mesh: Mesh,
}

impl Crosshair {
    pub fn new(gl: &gl::Gl, assets: &Assets) -> Result<Crosshair, failure::Error> {
        let program = Program::load(
            gl,
            assets,
            vec!["shaders/simple.vert", "shaders/simple.frag"],
        )?;
        program.set_used();
        let model_uniform = program.get_uniform("Model");
        let view_uniform = program.get_uniform("View");
        let projection_uniform = program.get_uniform("Projection");
        let mut vertices = Vec::new();
        let color = (0.0, 1.0, 1.0).into();
        let normal = (0.0, 0.0, 0.0).into();
        let scale = 1.0;
        let size = 10.0;
        let size_double = size * 2.0;
        let lights = (1.0, 1.0, 0.0).into();

        vertices.push(Vertex {
            position: (-size * scale, -size_double * scale, 0.0).into(),
            color,
            normal,
            lights,
        });
        vertices.push(Vertex {
            position: (size * scale, -size_double * scale, 0.0).into(),
            color,
            normal,
            lights,
        });
        vertices.push(Vertex {
            position: (size * scale, -size_double * scale, 0.0).into(),
            color,
            normal,
            lights,
        });
        vertices.push(Vertex {
            position: (size_double * scale, -size * scale, 0.0).into(),
            color,
            normal,
            lights,
        });
        vertices.push(Vertex {
            position: (size_double * scale, -size * scale, 0.0).into(),
            color,
            normal,
            lights,
        });
        vertices.push(Vertex {
            position: (size_double * scale, size * scale, 0.0).into(),
            color,
            normal,
            lights,
        });
        vertices.push(Vertex {
            position: (size_double * scale, size * scale, 0.0).into(),
            color,
            normal,
            lights,
        });
        vertices.push(Vertex {
            position: (size * scale, size_double * scale, 0.0).into(),
            color,
            normal,
            lights,
        });
        vertices.push(Vertex {
            position: (size * scale, size_double * scale, 0.0).into(),
            color,
            normal,
            lights,
        });
        vertices.push(Vertex {
            position: (-size * scale, size_double * scale, 0.0).into(),
            color,
            normal,
            lights,
        });
        vertices.push(Vertex {
            position: (-size * scale, size_double * scale, 0.0).into(),
            color,
            normal,
            lights,
        });
        vertices.push(Vertex {
            position: (-size_double * scale, size * scale, 0.0).into(),
            color,
            normal,
            lights,
        });
        vertices.push(Vertex {
            position: (-size_double * scale, size * scale, 0.0).into(),
            color,
            normal,
            lights,
        });
        vertices.push(Vertex {
            position: (-size_double * scale, -size * scale, 0.0).into(),
            color,
            normal,
            lights,
        });
        vertices.push(Vertex {
            position: (-size_double * scale, -size * scale, 0.0).into(),
            color,
            normal,
            lights,
        });
        vertices.push(Vertex {
            position: (-size * scale, -size_double * scale, 0.0).into(),
            color,
            normal,
            lights,
        });

        let mesh = Mesh::new(gl, &vertices).unwrap();
        Ok(Crosshair {
            gl: gl.clone(),
            mesh,
            program,
            model_uniform,
            view_uniform,
            projection_uniform,
        })
    }

    pub fn render(&self, model: Mat4, camera: &OrthographicCamera) {
        self.program.set_used();
        if let Some(uniform) = &self.model_uniform {
            uniform.set_uniform_matrix_4fv(&model);
        }
        if let Some(uniform) = &self.view_uniform {
            uniform.set_uniform_matrix_4fv(&Mat4::identity());
        }
        if let Some(uniform) = &self.projection_uniform {
            uniform.set_uniform_matrix_4fv(camera.get_projection());
        }
        unsafe {
            self.gl.Disable(gl::CULL_FACE);
            self.gl.Disable(gl::DEPTH_TEST);
            self.gl.Disable(gl::BLEND);
        }
        self.mesh.render_lines(&self.gl);
    }
}
