pub mod encoder {
    use std::io::Write;
    use std::process::{Command, Stdio};
    use std::time::Instant;
    use image::{self, ImageBuffer};

    pub fn encode<'a>(rgb_img: &ImageBuffer<image::Rgb<u8>, Vec<u8>>) -> (u32, u32, Vec<u8>, std::time::Duration) {
        let start_total = Instant::now();

        let (width, height) = rgb_img.dimensions();
        let dimensions_duration = Instant::now().duration_since(start_total);
        println!("Time to get dimensions: {:?}", dimensions_duration);

        // Convert the ImageBuffer to a Vec<u8> that can be passed to ffmpeg
        let raw_rgb = rgb_img.as_raw();
        let start_encode = Instant::now();

        // Set up the ffmpeg command for encoding
        let mut ffmpeg_process = Command::new("ffmpeg")
            .arg("-f").arg("rawvideo")
            .arg("-pixel_format").arg("rgb24")
            .arg("-video_size").arg(format!("{}x{}", width, height))
            .arg("-i").arg("pipe:0")
            .arg("-f").arg("h264")
            .arg("pipe:1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to start ffmpeg process");

        // Write the raw RGB data to ffmpeg's stdin
        ffmpeg_process.stdin.as_mut().unwrap().write_all(&raw_rgb).expect("Failed to write to stdin");

        // Read the encoded frames from ffmpeg's stdout
        let output = ffmpeg_process.wait_with_output().expect("Failed to read ffmpeg output");
        let encoded_frames = output.stdout;

        let encode_duration = start_encode.elapsed();
        println!("Time to encode frame: {:?}", encode_duration);

        let total_duration = Instant::now().duration_since(start_total);
        println!("Total time: {:?}", total_duration);

        (width, height, encoded_frames, encode_duration)
    }
}