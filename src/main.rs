#[macro_use]
extern crate lazy_static;

use windows_capture::{
    capture::WindowsCaptureHandler,
    frame::Frame,
    graphics_capture_api::InternalCaptureControl,
    settings::{ColorFormat, Settings},
    monitor::Monitor,
};

use anyhow::Result;

mod ecal_rs;

use ecal::format::prost::Prost;

type Publisher<T> = ecal::prost::Publisher<T>;

lazy_static! {
    static ref PUB: ecal::Publisher<ecal_rs::Frame, Prost<ecal_rs::Frame>> = {
        let mut publisher = Publisher::<ecal_rs::Frame>::new("/frame").unwrap();
        publisher
    };
}

struct Capture;


impl WindowsCaptureHandler for Capture {
    // To Get The Message From The Settings
    type Flags = String;

    // To Redirect To CaptureControl Or Start Method
    type Error = Box<dyn std::error::Error + Send + Sync>;

    // Function That Will Be Called To Create The Struct The Flags Can Be Passed
    // From `WindowsCaptureSettings`
    fn new(message: Self::Flags) -> Result<Self, Self::Error> {
        println!("Got The Flag: {message}");

        Ok(Self {})
    }

    // Called Every Time A New Frame Is Available
    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {

        let mut fb = frame.buffer()?;

        let mut screen = ecal_rs::Frame { height: fb.height(), 
                                          width: fb.width(),
                                          row_pitch: fb.row_pitch(),
                                          depth_pitch: fb.depth_pitch(),
                                          pixel_data: fb.as_raw_nopadding_buffer().unwrap().to_vec()
                                        };

        PUB.send(&screen)?;

        Ok(())
    }

    // Called When The Capture Item Closes Usually When The Window Closes, Capture
    // Session Will End After This Function Ends
    fn on_closed(&mut self) -> Result<(), Self::Error> {
        println!("Capture Session Closed");

        Ok(())
    }
}

fn main() {

    let _cal = ecal::Cal::new("frame");

    // Checkout Docs For Other Capture Items
    let foreground_window = Monitor::primary().expect("No Active Window Found");

    let settings = Settings::new(
        // Item To Captue
        foreground_window,
        // Capture Cursor
        Some(false),
        // Draw Borders (None Means Default Api Configuration)
        Some(false),
        // Kind Of Pixel Format For Frame To Have
        ColorFormat::Rgba8,
        // Will Be Passed To The New Function
        "It Works".to_string(),
    )
    .unwrap();

    // Every Error From `new`, `on_frame_arrived` and `on_closed` Will End Up Here
    Capture::start(settings).unwrap();
}