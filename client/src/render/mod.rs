mod block_mesh;
mod block_mesher;
mod block_render;
mod crosshair;
mod deferred;
mod skydome;
mod worldmesher;

use crate::*;
pub use block_mesh::{BlockMesh, BlockVertex};
pub use block_mesher::BlockMesher;
pub use block_render::BlockRenderer;
use common::chunk::*;
pub use crosshair::Crosshair;
pub use deferred::DeferredPipeline;
use gamework::video::*;
pub use skydome::SkyDome;
pub use worldmesher::WorldMesher;

pub struct Rendering {
    pub camera: PerspectiveCamera,
    underwater_color: Vec3,
    fog_distance: f32,
    pub render_lines: bool,
    fog_active: bool,
    crosshair: Crosshair,
    skydome: SkyDome,
    pub world_mesher: WorldMesher,
    deferred: DeferredPipeline,
}

impl Rendering {
    pub fn new(data: &mut GameContext, context: &mut SystemContext) -> Self {
        let crosshair = Crosshair::new(&context.video().gl(), &context.assets()).unwrap();
        let skydome = SkyDome::new(&context.video().gl(), &context.assets());
        let deferred = DeferredPipeline::new(
            context.video().width(),
            context.video().height(),
            &context.video().gl(),
            &context.assets(),
        )
        .unwrap();
        let world_mesher = WorldMesher::new(data.block_registry.clone());
        Rendering {
            camera: PerspectiveCamera::new(
                data.starting_position,
                0.0,
                std::f32::consts::FRAC_PI_2,
                context.video().aspect_ratio(),
                std::f32::consts::PI / 3.0,
                0.01,
                5000.0,
            ),
            underwater_color: Vec3::new(0.005, 0.02, 0.2),
            fog_distance: CHUNK_SIZE as f32 * data.config.render_range_chunks as f32,
            render_lines: false,
            fog_active: true,
            crosshair,
            skydome,
            world_mesher,
            deferred,
        }
    }

    pub fn resize(&mut self, _data: &mut GameContext, context: &mut SystemContext) {
        info!("Window resized, recreating deferred pipeline");
        self.deferred = DeferredPipeline::new(
            context.video().width(),
            context.video().height(),
            context.video().gl(),
            context.assets(),
        )
        .unwrap();
    }

    pub fn camera(&self) -> &PerspectiveCamera {
        &self.camera
    }

    pub fn toggle_render_lines(&mut self) {
        self.render_lines = !self.render_lines;
    }

    pub fn toggle_render_fog(&mut self) {
        self.fog_active = !self.fog_active;
    }

    pub fn render(
        &mut self,
        data: &mut GameContext,
        context: &mut SystemContext,
        in_water: bool,
        out_of_range: &mut HashSet<ChunkColumnPos>,
    ) {
        // Render geometry to frame buffer
        self.deferred.bind(context.video().gl());

        context.video().clear_screen();

        let max_range = data.world().render_range as i16;
        let center_col = data.world().center_col.clone();

        data.block_renderer.as_mut().unwrap().render(
            context.video().gl(),
            Mat4::identity(),
            &self.camera,
            max_range,
            &center_col,
            self.render_lines,
            out_of_range,
        );

        self.deferred.unbind(context.video().gl());

        let (render_back_color, fog_start, fog_end) = if in_water {
            (
                self.underwater_color,
                0.0,
                if self.fog_active {
                    self.fog_distance * 0.75
                } else {
                    1024.0
                },
            )
        } else {
            let fog_dist = if self.fog_active {
                self.fog_distance
            } else {
                1024.0
            };
            (
                data.daynight.get_fog_color(),
                fog_dist * 0.5,
                fog_dist * 0.9,
            )
        };

        // Actual rendering to screen is done in sRGB color space
        self.deferred.render_to_screen(
            context.video().gl(),
            &self.camera,
            context.video().width(),
            context.video().height(),
            &data.daynight.get_light_angle(),
            &data.daynight.get_light_color(),
            &render_back_color,
            fog_start,
            fog_end,
        );

        self.skydome.render(
            context.video().gl(),
            glm::translation(&self.camera.position),
            &self.camera,
            data.daynight.get_fog_color(),
            data.daynight.get_sky_color(),
            &data.daynight.get_light_angle(),
            &data.daynight.get_sun_color(),
        );

        // Render translucency framebuffer to screen with lighting and blending
        data.block_renderer.as_mut().unwrap().render_translucent(
            context.video().gl(),
            Mat4::identity(),
            &self.camera,
            &data.daynight.get_light_angle(),
            &data.daynight.get_light_color(),
            &render_back_color,
            fog_start,
            fog_end,
        );

        // Render gui elements on top
        let half_width = context.video().width() as f32 / 2.0;
        let half_height = context.video().height() as f32 / 2.0;
        self.crosshair.render(
            context.video().gl(),
            glm::translation(&Vec3::new(half_width, half_height, 0.0)),
            context.video().ui_camera(),
        );
    }
}
