extern crate nalgebra_glm as glm;

use std::path::PathBuf;

use common::block::*;
pub use gamework::glow;
use gamework::video::color::ColorRGBA;
use gamework::video::*;
use gamework::*;
use image::save_buffer_with_format;
use log::*;

pub fn generate_block_previews(
    block_registry_path: PathBuf,
    image_files_path: PathBuf,
    preview_files_path: PathBuf,
) {
    let preview_size = 48;
    let assets = Assets::default();
    App::run(
        "Block Previews",
        preview_size,
        preview_size,
        true,
        3,
        3,
        Box::new(DummyState {
            blocks: None,
            sprite_batch: None,
            current_block: 0,
        }),
        &assets,
        DummyContext {
            preview_size,
            block_registry_path,
            image_files_path,
            preview_files_path,
        },
        Box::new(setup),
    );
}

struct DummyContext {
    pub block_registry_path: PathBuf,
    pub image_files_path: PathBuf,
    pub preview_files_path: PathBuf,
    pub preview_size: u32,
}

impl SharedContext for DummyContext {
    fn initialize(&mut self, _context: &mut SystemContext) {}
}

struct DummyState {
    blocks: Option<BlockRegistry>,
    sprite_batch: Option<SpriteBatcher>,
    current_block: u32,
}

impl State<DummyContext> for DummyState {
    fn initialize(&mut self, context: &mut DummyContext, system: &mut SystemContext) {
        info!("Loading block registry");
        self.blocks = Some(BlockRegistry::load_or_create(&context.block_registry_path).unwrap());

        info!(
            "Setting up render target of {} x {}",
            context.preview_size, context.preview_size
        );

        system
            .video_mut()
            .set_background_color(&ColorRGBA::new(0.0, 0.0, 0.0, 0.0));

        // Set up rendering for the cube preview
        let texture_atlas = TextureAtlas::load_directory(
            &context.image_files_path,
            system.video().gl(),
            TextureFilter::Nearest,
        );
        info!(
            "{} block textures loaded into atlas from {}",
            texture_atlas.frames().len(),
            context.image_files_path.to_str().unwrap()
        );
        let batch = SpriteBatcher::new(system.video().gl(), texture_atlas);
        self.sprite_batch = Some(batch);
    }

    fn update(
        &mut self,
        _delta: f32,
        _context: &mut DummyContext,
        _gui: &egui::Context,
        _input_events: &Vec<InputEvent>,
        _system: &mut SystemContext,
    ) -> StateCommand<DummyContext> {
        if self.current_block >= self.blocks.as_ref().unwrap().all_blocks().len() as u32 {
            info!("Done");
            StateCommand::CloseState
        } else {
            StateCommand::None
        }
    }

    fn resize(&mut self, _data: &mut DummyContext, _context: &mut SystemContext) {}

    fn render(&mut self, context: &mut DummyContext, system: &mut SystemContext) {
        // Render 3 sides of a fake cube
        let width = context.preview_size as f32;
        let half_width = width / 2.0;
        let height = context.preview_size as f32;
        let height_1 = height - height * 0.25;
        let height_2 = height - height * 0.5;
        let height_3 = height - height * 0.75;
        let x1 = half_width;
        let y1 = height;
        let x2 = width;
        let y2 = height_1;
        let x3 = half_width;
        let y3 = height_2;
        let x4 = 0.0;
        let y4 = height_1;
        let x5 = 0.0;
        let y5 = height_3;
        let x6 = half_width;
        let y6 = 0.0;
        let x7 = width;
        let y7 = height_3;

        let def = self
            .blocks
            .as_ref()
            .unwrap()
            .get(self.current_block)
            .clone();

        let (color_up, color_front, color_side) = if def.light > 0 {
            (ColorRGBA::white(), ColorRGBA::white(), ColorRGBA::white())
        } else {
            (
                ColorRGBA::new(0.9, 0.9, 0.9, 1.0),
                ColorRGBA::new(0.5, 0.5, 0.5, 1.0),
                ColorRGBA::new(0.2, 0.2, 0.2, 1.0),
            )
        };
        if def.textures.len() != 6 {
            info!("Skipping preview for {} - no 6 textures", def.name);
        } else {
            info!("Generating preview for {}", def.name);
            if let Some(batch) = self.sprite_batch.as_mut() {
                let texture_id_up = batch
                    .texture_atlas()
                    .find_id(&def.textures[FACE_ZP])
                    .unwrap();
                batch.add_points(
                    x1,
                    y1,
                    x4,
                    y4 - 0.25,
                    x3,
                    y3 - 0.25,
                    x2,
                    y2 - 0.25,
                    &color_up,
                    texture_id_up,
                );
                let texture_id_front = batch
                    .texture_atlas()
                    .find_id(&def.textures[FACE_XP])
                    .unwrap();
                batch.add_points(
                    x3,
                    y3,
                    x4,
                    y4,
                    x5,
                    y5,
                    x6,
                    y6,
                    &color_front,
                    texture_id_front,
                );
                let texture_id_right = batch
                    .texture_atlas()
                    .find_id(&def.textures[FACE_YP])
                    .unwrap();
                batch.add_points(
                    x2,
                    y2,
                    x3,
                    y3,
                    x6,
                    y6,
                    x7,
                    y7,
                    &color_side,
                    texture_id_right,
                );
                batch.draw(
                    system.video().gl(),
                    system.video().ui_camera().get_projection(),
                );
            }
            // Get the rendered result from the framebuffer and write it to a file
            unsafe {
                let mut pixel_bytes =
                    vec![0; (context.preview_size * context.preview_size * 4) as usize];
                system.video().gl().read_pixels(
                    0,
                    0,
                    context.preview_size as i32,
                    context.preview_size as i32,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    PixelPackData::Slice(&mut pixel_bytes),
                );
                save_buffer_with_format(
                    context
                        .preview_files_path
                        .join(format!("block_preview_{}.png", def.code)),
                    &pixel_bytes,
                    context.preview_size,
                    context.preview_size,
                    image::ColorType::Rgba8,
                    image::ImageFormat::Png,
                )
                .unwrap();
            }
        }
        self.current_block += 1;
    }

    fn shutdown(&mut self) {}

    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

fn setup(_context: &mut DummyContext, _system: &mut SystemContext, _gui: &mut egui::Context) {}
