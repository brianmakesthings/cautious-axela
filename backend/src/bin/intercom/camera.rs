use nokhwa::*;

pub fn mainloop() {
    match Camera::new(
        0,
        Some(CameraFormat::new_from(1280, 720, FrameFormat::MJPEG, 30)),
    ) {
        Ok(camera) => {
            println!("Using camera {}", camera.info());

            let mut camera = camera;
            camera.open_stream().unwrap();

            loop {
                let frame = camera.frame_raw().unwrap();
                // todo: send frame to web
                // probably by udp (must split into chunks)
            }
        }
        Err(err) => {
            println!("{}", err);
        }
    }
}
