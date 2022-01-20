use crate::profile::Profile;
use crate::*;
use crate::{audio::*, video::*};
use gl;
use log::*;

pub const PROFILE_SAMPLES: usize = 300;

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
    pub fn new(gl: &gl::Gl, width: u32, height: u32, dpi: f32, assets: &Assets) -> SystemContext {
        // Initialize audio and video subsystem
        let video = Video::new(gl.clone(), width, height, dpi);
        let audio = AudioOutput::default();

        info!("Context initialization complete");
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

    pub fn assets(&self) -> &Assets {
        &self.assets
    }

    pub fn input_mut(&mut self) -> &mut Input {
        &mut self.input
    }

    pub fn input(&self) -> &Input {
        &self.input
    }

    pub fn play_sound(&mut self, sound: &AudioSource) {
        self.audio.play_source(sound);
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

    pub fn fill_profile_buffer(&self, buffer: &mut [[f32; 4]; PROFILE_SAMPLES]) {
        let frame_iter = self.frame_profile.duration_buffer.iter();
        let mut update_iter = self.update_profile.duration_buffer.iter();
        let mut render_iter = self.render_profile.duration_buffer.iter();
        let mut swap_iter = self.swap_profile.duration_buffer.iter();
        let mut index = 0;
        for frame_duration in frame_iter {
            let update_duration = update_iter.next().unwrap();
            let render_duration = render_iter.next().unwrap();
            let swap_duration = swap_iter.next().unwrap();
            buffer[index][0] = frame_duration - update_duration - render_duration - swap_duration;
            buffer[index][1] = *update_duration;
            buffer[index][2] = *render_duration;
            buffer[index][3] = *swap_duration;
            index += 1;
        }
    }
}
