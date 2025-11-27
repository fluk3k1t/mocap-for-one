use std::{thread, time::Duration};

use eframe::egui;
use opencv::{core::{Mat, MatTraitConst, MatTraitConstManual}, videoio::{VideoCapture, VideoCaptureTrait}};
use std::sync::{Arc, Mutex};

struct App {
    img: Arc<Mutex<Option<egui::ColorImage>>>,  
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // if let Err(e) = self.render_cam(ui) {
            //     ui.label(format!("Error capturing frame: {}", e));
            // }
            let img_lock = self.img.lock().unwrap();
            if let Some(ref img) = *img_lock {
                let texture = ui.ctx().load_texture("cam_frame", img.clone(), Default::default());
                ui.image(&texture);
                println!("Rendered frame");
            } else {
                println!("No frame available yet");
            }

            thread::sleep(Duration::from_millis(1000/60));
        });

        ctx.request_repaint();
    }
}

impl App {
    fn new(img: Arc<Mutex<Option<egui::ColorImage>>>) -> Self {
        Self {
            img,
        }
    }

    fn render_cam(&mut self, ui: &mut egui::Ui) -> opencv::Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

fn main() -> eframe::Result {

    let img = Arc::new(Mutex::new(None));
    let img_clone = img.clone();
    
    thread::spawn(move || {
        let mut cam = VideoCapture::new(0, opencv::videoio::CAP_ANY).unwrap();
        let mut last_time_millis = std::time::Instant::now();

        loop {
            // show time to capture frame
            let sep = std::time::Instant::now().duration_since(last_time_millis).as_millis();
            println!("Time to capture frame: {} ms", std::time::Instant::now().duration_since(last_time_millis).as_millis());

            let mut frame = Mat::default();
            if cam.read(&mut frame).is_ok() {
                let mut rgb = Mat::default();
                opencv::imgproc::cvt_color(&frame, &mut rgb, opencv::imgproc::COLOR_BGR2RGB, 0, opencv::core::AlgorithmHint::ALGO_HINT_DEFAULT).unwrap();
                let mut scaled = Mat::default();
                opencv::imgproc::resize(&rgb, &mut scaled, opencv::core::Size { width: 320, height: 240 }, 0.0, 0.0, opencv::imgproc::INTER_CUBIC).unwrap();
                frame = scaled;
            }

            let frame_size = [frame.cols() as usize, frame.rows() as usize];
            let img  = egui::ColorImage::from_rgb(frame_size, frame.data_bytes().unwrap());
            // let texture = ui.ctx().load_texture("cam_frame", img, Default::default());
            // ui.image(&texture);

            let mut img_lock = img_clone.lock().unwrap();
            *img_lock = Some(img);

            // last_time_millis = std::time::Instant::now();
        }
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_native(
        "MocapForOne",
        options,
        Box::new(|_cc| Ok(Box::new(App::new(img)))),
    )
}