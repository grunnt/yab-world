use failure;
use gamework::glow::*;
use gamework::video::*;
use gamework::*;
use nalgebra_glm::*;
pub struct Crosshair {
    program: ShaderProgram,
    model_uniform: Option<UniformLocation>,
    view_uniform: Option<UniformLocation>,
    projection_uniform: Option<UniformLocation>,
    mesh: Mesh,
}

impl Crosshair {
    pub fn new(gl: &glow::Context, assets: &Assets) -> Result<Crosshair, failure::Error> {
        let program = ShaderProgram::load(
            gl,
            assets,
            "shaders/simple.vert",
            "shaders/simple.frag",
            "simple".to_string(),
        )?;
        program.set_used(gl);
        let model_uniform = program.get_uniform(gl, "Model");
        let view_uniform = program.get_uniform(gl, "View");
        let projection_uniform = program.get_uniform(gl, "Projection");
        let mut vertices = Vec::new();
        let color = (1.0, 1.0, 1.0).into();
        let size = 10.0;

        vertices.push(Vertex {
            position: (-size, 0.0, 0.0).into(),
            color,
        });
        vertices.push(Vertex {
            position: (size, 0.0, 0.0).into(),
            color,
        });
        vertices.push(Vertex {
            position: (0.0, -size, 0.0).into(),
            color,
        });
        vertices.push(Vertex {
            position: (0.0, size, 0.0).into(),
            color,
        });

        let mesh = Mesh::new(gl, &vertices);
        Ok(Crosshair {
            mesh,
            program,
            model_uniform,
            view_uniform,
            projection_uniform,
        })
    }

    pub fn render(&self, gl: &glow::Context, model: Mat4, camera: &OrthographicCamera) {
        self.program.set_used(gl);
        if let Some(uniform) = &self.model_uniform {
            self.program.set_uniform_matrix_4fv(gl, &uniform, &model);
        }
        if let Some(uniform) = &self.view_uniform {
            self.program
                .set_uniform_matrix_4fv(gl, &uniform, &Mat4::identity());
        }
        if let Some(uniform) = &self.projection_uniform {
            self.program
                .set_uniform_matrix_4fv(gl, &uniform, camera.get_projection());
        }
        unsafe {
            gl.disable(glow::CULL_FACE);
            gl.disable(glow::DEPTH_TEST);
            gl.disable(glow::BLEND);
        }
        self.mesh.render_lines(&gl);
    }
}
