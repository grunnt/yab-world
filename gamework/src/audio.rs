use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::{io::Cursor, sync::Arc};

// Based on https://github.com/bevyengine/bevy/blob/master/crates/bevy_audio/src/audio_source.rs

#[derive(Debug, Clone)]
pub struct AudioSource {
    pub bytes: Arc<Vec<u8>>,
}

impl AsRef<[u8]> for AudioSource {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl AudioSource {
    pub fn load(path: &Path) -> Self {
        let mut f = File::open(&path).expect("no file found");
        let metadata = fs::metadata(&path).expect("unable to read metadata");
        let mut buffer = vec![0; metadata.len() as usize];
        f.read(&mut buffer).expect("buffer overflow");
        AudioSource {
            bytes: Arc::new(buffer),
        }
    }
}

pub trait Decodable: Send + Sync + 'static {
    type Decoder;

    fn decoder(&self) -> Self::Decoder;
}

impl Decodable for AudioSource {
    type Decoder = rodio::Decoder<Cursor<AudioSource>>;

    fn decoder(&self) -> Self::Decoder {
        rodio::Decoder::new(Cursor::new(self.clone())).unwrap()
    }
}

pub struct AudioOutput {
    pub volume: f32,
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sounds: HashMap<String, AudioSource>,
}

impl Default for AudioOutput {
    fn default() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();

        Self {
            volume: 1.0,
            _stream: stream,
            stream_handle,
            sounds: HashMap::new(),
        }
    }
}

impl AudioOutput {
    pub fn load_sound(&mut self, path: &Path) {
        let sound_name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let sound = AudioSource::load(path);
        self.sounds.insert(sound_name, sound);
    }

    pub fn play_sound(&self, sound_name: &str) {
        if let Some(source) = self.sounds.get(sound_name) {
            self.play_source(source);
        }
    }

    pub fn play_source(&self, audio_source: &AudioSource) {
        let sink = Sink::try_new(&self.stream_handle).unwrap();
        sink.set_volume(self.volume);
        sink.append(audio_source.decoder());
        sink.detach();
    }
}
