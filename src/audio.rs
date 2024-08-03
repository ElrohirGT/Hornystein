use std::{fs::File, io::BufReader};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

pub struct AudioPlayer {
    _device_stream: (OutputStream, OutputStreamHandle),
    pub background: Track,
    pub loose_song: Track,
    pub win_song: Track,
}

pub struct Track {
    path: String,
    pub sink: Sink,
}

impl Track {
    pub fn new(track_path: String, stream_handle: &OutputStreamHandle) -> Self {
        let sink = Sink::try_new(stream_handle).unwrap();
        Track {
            path: track_path,
            sink,
        }
    }

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
        let file_path = format!("{}sounds/{}", asset_dir, "background.mp3");
        let background = Track::new(file_path, &stream_handle);

        let file_path = format!("{}sounds/{}", asset_dir, "win.mp3");
        let win_song = Track::new(file_path, &stream_handle);

        let file_path = format!("{}sounds/{}", asset_dir, "loose.mp3");
        let loose_song = Track::new(file_path, &stream_handle);

        let _device_stream = (stream, stream_handle);
        AudioPlayer {
            _device_stream,
            background,
            win_song,
            loose_song,
        }
    }
}
