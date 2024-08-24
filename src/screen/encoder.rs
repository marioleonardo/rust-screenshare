pub mod encoder {
    use openh264::{encoder::Encoder, formats::{RgbSliceU8, YUVBuffer}};
    use std::time::Instant;
    use image::{self, ImageBuffer, Rgb};

    pub fn encode<'a>(rgb_img: &ImageBuffer<image::Rgb<u8>, Vec<u8>>) -> (u32, u32, Vec<u8>, std::time::Duration) {
        let start_total = Instant::now();
        
        let (width, height) = rgb_img.dimensions();
        let dimensions_duration = Instant::now().duration_since(start_total);
        println!("Time to get dimensions: {:?}", dimensions_duration);

        let start_conversion = Instant::now();
        let yuv_buffer = YUVBuffer::from_rgb_source(RgbSliceU8::new(&rgb_img, (width as usize, height as usize)));
        let conversion_duration = Instant::now().duration_since(start_conversion);
        println!("Time to convert RGB to YUV: {:?}", conversion_duration);
        
        let start_encoder_creation = Instant::now();
        let mut encoder = Encoder::new().unwrap();
        let encoder_creation_duration = Instant::now().duration_since(start_encoder_creation);
        println!("Time to create encoder: {:?}", encoder_creation_duration);

        let start_encode = Instant::now();
        let encoded_frames = encoder.encode(&yuv_buffer).expect("Failed to encode frame").to_vec();
        let encode_duration = start_encode.elapsed();
        println!("Time to encode frame: {:?}", encode_duration);

        let total_duration = Instant::now().duration_since(start_total);
        println!("Total time: {:?}", total_duration);

        (width, height, encoded_frames, encode_duration)
    }
}
