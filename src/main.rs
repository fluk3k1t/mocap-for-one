use eframe::egui;
use opencv::{core, highgui, imgproc, prelude::*, videoio};
use parking_lot::Mutex;
use std::time::Instant;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    thread,
};

use mocap_for_one::*;

/// Desired display size (smaller = faster)
const DISPLAY_W: i32 = 320;
const DISPLAY_H: i32 = 240;

fn main() -> Result<(), eframe::Error> {
    // let options = eframe::NativeOptions {
    //     viewport: egui::ViewportBuilder::default()
    //         .with_inner_size([640.0, 480.0])
    //         .with_resizable(false),
    //     vsync: false,
    //     multisampling: 0,
    //     ..Default::default()
    // };

    // eframe::run_native(
    //     "OpenCV + eGUI App (High FPS)",
    //     options,
    //     Box::new(|_cc| Ok(Box::new(MyApp::new()))),
    // )

    let pipeline0: PipeLine<(), String> = PipeLine::new(|inner| {
        loop {
            let inner: std::sync::MutexGuard<'_, PipeLineInternal<(), String>> = inner.lock().unwrap();
            inner.o.send("Hello from pipeline!".to_string()).unwrap();
        }
    });

    let pipeline1: PipeLine<String, ()> = PipeLine::new(|inner| {
        loop {
            let inner: std::sync::MutexGuard<'_, PipeLineInternal<String, ()>> = inner.lock().unwrap();
            if let Some(receiver) = &inner.i {
                if let Ok(msg) = receiver.recv() {
                    println!("Received message: {}", msg);
                    // println!("time {}", Instant::now().elapsed().as_millis());
                }
            }
        }
    });

    let _joined_pipeline = join(pipeline0, pipeline1);

    loop {
        println!("Main thread alive");
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Ok(())
}

struct MyApp {
    // UI state
    last_fps_time: Instant,
    fps: f64,
    frame_count: u64,

    // Shared frame storage
    frame_image: Arc<Mutex<Option<egui::ColorImage>>>,
    new_frame_flag: Arc<AtomicBool>,
    frame_id: Arc<AtomicU64>,

    // Texture for egui
    texture: Option<egui::TextureHandle>,
    last_frame_id: u64,
}

impl MyApp {
    fn new() -> Self {
        let frame_image = Arc::new(Mutex::new(None));
        let frame_image_clone = Arc::clone(&frame_image);

        let new_frame_flag = Arc::new(AtomicBool::new(false));
        let new_frame_flag_clone = Arc::clone(&new_frame_flag);

        let frame_id = Arc::new(AtomicU64::new(0));
        let frame_id_clone = Arc::clone(&frame_id);

        // Spawn camera thread
        thread::spawn(move || {
            // open camera
            let mut cam = match videoio::VideoCapture::new(0, videoio::CAP_ANY) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("failed to open camera: {}", e);
                    return;
                }
            };

            if !videoio::VideoCapture::is_opened(&cam).unwrap_or(false) {
                eprintln!("camera not opened");
                return;
            }

            // try to set properties
            let _ = cam.set(videoio::CAP_PROP_BUFFERSIZE, 1.0);
            let _ = cam.set(videoio::CAP_PROP_FPS, 60.0);

            // optional OpenCV window (can remove for headless)
            let window_name = "OpenCV Camera (optional)";
            let _ = highgui::named_window(window_name, highgui::WINDOW_AUTOSIZE);

            let mut frame_id_local: u64 = 0;

            loop {
                let mut frame = Mat::default();
                // blocking read — many backends block until next frame
                if cam.read(&mut frame).is_ok() {
                    // optionally show in OpenCV window (for debugging)
                    let _ = highgui::imshow(window_name, &frame);
                    let _ = highgui::wait_key(1);

                    // Resize to display size (do this on camera thread)
                    let mut small = Mat::default();
                    let size = core::Size::new(DISPLAY_W, DISPLAY_H);
                    let _ =
                        imgproc::resize(&frame, &mut small, size, 0.0, 0.0, imgproc::INTER_NEAREST);

                    // Convert BGR -> RGB
                    let mut rgb = Mat::default();
                    let _ = imgproc::cvt_color(&small, &mut rgb, imgproc::COLOR_BGR2RGB, 0, core::AlgorithmHint::ALGO_HINT_DEFAULT);

                    // Convert Mat -> Vec<u8>
                    match mat_to_rgb_bytes(&rgb) {
                        Ok(vec) => {
                            // Create ColorImage (this allocates once per frame but off main thread)
                            let color_image = egui::ColorImage::from_rgb(
                                [DISPLAY_W as usize, DISPLAY_H as usize],
                                &vec,
                            );
                            
                            // store into shared slot
                            {
                                let mut guard = frame_image_clone.lock();
                                *guard = Some(color_image);
                            }
                            frame_id_local += 1;
                            frame_id_clone.store(frame_id_local, Ordering::Release);
                            new_frame_flag_clone.store(true, Ordering::Release);
                        }
                        Err(e) => {
                            eprintln!("mat_to_rgb_bytes error: {}", e);
                        }
                    }
                } else {
                    // If read failed, sleep a bit to avoid busy spin
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
            }
        });

        Self {
            last_fps_time: Instant::now(),
            fps: 0.0,
            frame_count: 0,
            frame_image,
            new_frame_flag,
            frame_id,
            texture: None,
            last_frame_id: 0,
        }
    }
}

/// Helper: convert Mat (RGB) to Vec<u8>
fn mat_to_rgb_bytes(mat: &Mat) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let rows = mat.rows() as usize;
    let cols = mat.cols() as usize;
    let data = mat.data_bytes()?;
    if data.len() != rows * cols * 3 {
        return Err("unexpected mat data length".into());
    }
    Ok(data.to_vec()) // copy once here (done off UI thread)
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // FPS measurement for UI update rate
        self.frame_count += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_fps_time);
        if elapsed.as_secs_f64() >= 1.0 {
            self.fps = self.frame_count as f64 / elapsed.as_secs_f64();
            self.frame_count = 0;
            self.last_fps_time = now;
        }

        // Only request repaint if there is a new frame (reduces useless UI updates)
        if self.new_frame_flag.load(Ordering::Acquire) {
            ctx.request_repaint();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("OpenCV + eGUI (Optimized)");
            ui.label(format!("eGUI Update FPS: {:.1}", self.fps));
            ui.separator();

            // If there is a new frame, update the texture once
            if self
                .new_frame_flag
                .compare_exchange(true, false, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                // we observed a new frame: swap it into texture
                if let Some(img) = self.frame_image.lock().take() {
                    self.last_frame_id = self.frame_id.load(Ordering::Acquire);
                    // create or update texture
                    if let Some(tex) = &mut self.texture {
                        tex.set(img.clone(), egui::TextureOptions::NEAREST);
                    } else {
                        // load texture into egui
                        let handle = ui.ctx().load_texture(
                            "video_frame",
                            img.clone(),
                            egui::TextureOptions::NEAREST,
                        );
                        self.texture = Some(handle);
                    }
                }
            }

            // display texture if exists
            if let Some(texture) = &self.texture {
                let display_size = egui::Vec2::new(320.0, 240.0);
                ui.image(egui::ImageSource::Texture(egui::load::SizedTexture::new(
                    texture.id(),
                    display_size,
                )));
                ui.label(format!("Frame ID: {}", self.last_frame_id));
            } else {
                ui.label("Waiting for camera...");
            }

            ui.separator();
            if ui.button("Exit").clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }
}
