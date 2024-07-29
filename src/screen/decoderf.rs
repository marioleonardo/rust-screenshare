pub mod decoder {

    use std::process::{Command, Stdio};
    use std::io::Write;
    use std::time::Instant;
    use image::{ImageBuffer, Rgba};

    pub fn decode(encoded_frames: Vec<u8>, width: u32, height: u32) -> (std::time::Duration, ImageBuffer<Rgba<u8>, Vec<u8>>) {
        let start_decode = Instant::now();

        // Set up the ffmpeg command for decoding
        let mut ffmpeg_process = Command::new("ffmpeg")
            .arg("-f").arg("h264")
            .arg("-i").arg("pipe:0")
            .arg("-f").arg("rawvideo")
            .arg("-pix_fmt").arg("rgba")
            .arg("-video_size").arg(format!("{}x{}", width, height))
            .arg("pipe:1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start ffmpeg process");

        // Write the encoded frames to ffmpeg's stdin
        ffmpeg_process.stdin.as_mut().unwrap().write_all(&encoded_frames).expect("Failed to write to stdin");

        // Read the decoded RGBA frame from ffmpeg's stdout
        let output = ffmpeg_process.wait_with_output().expect("Failed to read ffmpeg output");
        let rgba_buffer = output.stdout;

        let decode_duration = start_decode.elapsed();
        println!("Time to decode frame: {:?}", decode_duration);

        // Debugging information
        let expected_size = (width * height * 4) as usize;
        let actual_size = rgba_buffer.len();
        println!("Expected buffer size: {}", expected_size);
        println!("Actual buffer size: {}", actual_size);

        // Ensure the buffer is of the expected size before creating ImageBuffer
        if actual_size != expected_size {
            panic!("Buffer size does not match expected size! Expected: {}, Actual: {}", expected_size, actual_size);
        }

        // Create ImageBuffer from the RGBA buffer
        let out_img = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, rgba_buffer)
            .expect("Failed to create image buffer");

        (decode_duration, out_img)
    }
}