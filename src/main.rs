use std::io::BufWriter;

use fs_err::OpenOptions;
use simplelog::{CombinedLogger, LevelFilter, TermLogger, WriteLogger};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[allow(clippy::single_match)]
#[allow(clippy::collapsible_match)]
fn main() -> anyhow::Result<()> {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Warn,
            Default::default(),
            simplelog::TerminalMode::Mixed,
            simplelog::ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Trace,
            Default::default(),
            BufWriter::new(
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("ignore.log")?,
            ),
        ),
    ])?;

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            window_id, event, ..
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            _ => {}
        },
        _ => {}
    })
}
