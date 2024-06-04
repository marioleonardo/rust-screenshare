pub mod encoder{

    use openh264::{decoder::Decoder, encoder::Encoder, formats::{RGBSource, RgbSliceU8, YUVBuffer, YUVSource}};
    use scap::{
        capturer::{Area, Capturer, Options, Point, Size},
        frame::{BGRAFrame, Frame, FrameType, YUVFrame},
    };
    use std::time::Instant;
    use image::{self, DynamicImage, GenericImageView, ImageBuffer, Rgba};
    pub fn encode<'a>(rgb_img: &ImageBuffer<image::Rgb<u8>, Vec<u8>>) -> (u32, u32, Vec<u8>, std::time::Duration) {
    
        let (width, height) = rgb_img.dimensions();
        let yuv_buffer = YUVBuffer::from_rgb_source( RgbSliceU8::new(&rgb_img, (width as usize, height as usize)));
        // Step 1.5: Convert the RGBA image to YUV
        // print!("dimensions: {}\n", yuv_buffer.u().len()*3);
    
        
        // Step 2: Encode the image into video frames
        let mut encoder = Encoder::new().unwrap();
    
        let start_encode = Instant::now();
        let encoded_frames = encoder.encode(&yuv_buffer).expect("Failed to encode frame").to_vec();
        // print!("dimensions: {}\n",  encoded_frames.to_vec().len());
    
        let encode_duration = start_encode.elapsed();
        (width, height, encoded_frames, encode_duration)
    }
    
}