use std::{fs::File, io::BufReader};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

pub struct AudioPlayer {
    _device_stream: (OutputStream, OutputStreamHandle),
    pub background: Track,
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Audios {
    Background,
}

pub struct Track {
    path: String,
    pub sink: Sink,
}

impl Track {
    pub fn play(&self) {
        let path = BufReader::new(File::open(&self.path).unwrap());
        let source = Decoder::new(path).unwrap();
        self.sink.append(source)
    }
}

impl AudioPlayer {
    pub fn new(asset_dir: &str) -> Self {
        let (stream, stream_handle) =
            OutputStream::try_default().expect("Can't get default output stream!");
        let background_file_path = format!("{}sounds/{}", asset_dir, "background.mp3");
        let sink = Sink::try_new(&stream_handle).unwrap();
        let background = Track {
            path: background_file_path,
            sink,
        };

        let _device_stream = (stream, stream_handle);
        AudioPlayer {
            _device_stream,
            background,
        }
    }
}
