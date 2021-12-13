use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::io::Read;
use std::path::Path;
use std::{
    fs::{self, File},
    marker::PhantomData,
};
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

pub struct AudioOutput<P = AudioSource>
where
    P: Decodable,
{
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    phantom: PhantomData<P>,
}

impl<P> Default for AudioOutput<P>
where
    P: Decodable,
{
    fn default() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();

        Self {
            _stream: stream,
            stream_handle,
            phantom: PhantomData,
        }
    }
}

impl<P> AudioOutput<P>
where
    P: Decodable,
    <P as Decodable>::Decoder: rodio::Source + Send + Sync,
    <<P as Decodable>::Decoder as Iterator>::Item: rodio::Sample + Send + Sync,
{
    pub fn play_source(&self, audio_source: &P) {
        let sink = Sink::try_new(&self.stream_handle).unwrap();
        sink.append(audio_source.decoder());
        sink.detach();
    }
}

// let bleep_file = std::fs::File::open(assets.assets_path("sounds/bleep.wav")).unwrap();
// pub bleep_source: SamplesBuffer<f32>,

//     let bleep_source = rodio::Decoder::new(BufReader::new(bleep_file))
//         .unwrap()
//         .;
