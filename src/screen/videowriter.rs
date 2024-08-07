use std::fs::{File, OpenOptions};
use std::io::Write;

pub struct VideoWriter {
    buffer: Vec<u8>,
    nal_units: usize,
    frame_limit: usize,
    file_path: String,
}

impl VideoWriter {
    pub fn new(frame_limit: usize, file_path: String) -> Self {
        Self {
            buffer: Vec::new(),
            nal_units: 0,
            frame_limit,
            file_path,
        }
    }

    pub fn add_frame(&mut self, nal_unit: Vec<u8>) {
        self.buffer.extend(nal_unit);
        self.nal_units += 1;
        if self.nal_units >= self.frame_limit {
            self.write_to_file();
            self.buffer.clear();
            self.nal_units = 0;
        }
    }

    pub fn write_to_file(&self) {
        let file_exists = std::path::Path::new(&self.file_path).exists();
        let mut file = if file_exists {
            OpenOptions::new().append(true).open(&self.file_path).unwrap()
        } else {
            File::create(&self.file_path).unwrap()
        };

        file.write_all(&self.buffer).unwrap();
    }
}
