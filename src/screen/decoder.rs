pub mod decoder{

    use openh264::decoder::Decoder;
    use std::time::Instant;
    use image::{ImageBuffer, Rgba};
    pub fn decode(encoded_frames: Vec<u8>, width: u32, height: u32) -> (std::time::Duration, ImageBuffer<Rgba<u8>, Vec<u8>>) {
        let mut decoder = Decoder::new().expect("Failed to create decoder");
    
        let start_decode = Instant::now();
        
        let decoded_frame = decoder.decode(&encoded_frames).expect("Failed to decode frame");
        let decode_duration = start_decode.elapsed();
        let mut rgba_buffer = vec![0u8; (width * height * 4) as usize];
        if decoded_frame.is_none(){
            return (decode_duration, ImageBuffer::<Rgba<u8>, Vec<u8>>::new(5, height));
        }
        decoded_frame.unwrap().write_rgba8(&mut rgba_buffer);
    
    
        // Convert the decoded frame to an ImageBuffer
        let out_img = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height,  rgba_buffer)
            .expect("Failed to create image buffer");
        
        (decode_duration, out_img)
    }

    
}