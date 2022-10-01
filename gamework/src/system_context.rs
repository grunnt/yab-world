use std::sync::Arc;

use crate::audio::{AudioOutput, AudioSource};
use crate::profile::Profile;
use crate::video::*;
use crate::*;

pub const PROFILE_SAMPLES: usize = 128;

pub struct SystemContext {
    video: Video,
    assets: Assets,
    audio: AudioOutput,
    input: Input,
    frame_profile: Profile,
    render_profile: Profile,
    swap_profile: Profile,
    update_profile: Profile,
}

impl SystemContext {
    pub fn new(
        gl: Arc<glow::Context>,
        width: u32,
        height: u32,
        dpi: f32,
        assets: &Assets,
    ) -> SystemContext {
        // Initialize audio and video subsystem
        let video = Video::new(gl, width, height, dpi);
        let audio = AudioOutput::default();
        SystemContext {
            video,
            assets: assets.clone(),
            audio,
            input: Input::new(),
            frame_profile: Profile::new(PROFILE_SAMPLES),
            render_profile: Profile::new(PROFILE_SAMPLES),
            swap_profile: Profile::new(PROFILE_SAMPLES),
            update_profile: Profile::new(PROFILE_SAMPLES),
        }
    }

    pub fn video_mut(&mut self) -> &mut Video {
        &mut self.video
    }

    pub fn video(&self) -> &Video {
        &self.video
    }

    pub fn audio_mut(&mut self) -> &mut AudioOutput {
        &mut self.audio
    }

    pub fn audio(&self) -> &AudioOutput {
        &self.audio
    }

    pub fn play_sound(&mut self, sound: &AudioSource) {
        self.audio.play_source(sound);
    }

    pub fn assets(&self) -> &Assets {
        &self.assets
    }

    pub fn input_mut(&mut self) -> &mut Input {
        &mut self.input
    }

    pub fn input(&self) -> &Input {
        &self.input
    }

    pub fn frame_profile_mut(&mut self) -> &mut Profile {
        &mut self.frame_profile
    }

    pub fn frame_profile(&self) -> &Profile {
        &self.frame_profile
    }

    pub fn render_profile_mut(&mut self) -> &mut Profile {
        &mut self.render_profile
    }

    pub fn render_profile(&self) -> &Profile {
        &self.render_profile
    }

    pub fn swap_profile_mut(&mut self) -> &mut Profile {
        &mut self.swap_profile
    }

    pub fn swap_profile(&self) -> &Profile {
        &self.swap_profile
    }

    pub fn update_profile_mut(&mut self) -> &mut Profile {
        &mut self.update_profile
    }

    pub fn update_profile(&self) -> &Profile {
        &self.update_profile
    }
}
