use std::io::BufWriter;

use anyhow::Context;
use fs_err::OpenOptions;
use log::error;
use pollster::block_on;
use simplelog::{CombinedLogger, LevelFilter, TermLogger, WriteLogger};
use wgpu::{
    Backends, Device, DeviceDescriptor, LoadOp, RenderPassColorAttachment, RenderPassDescriptor,
    RequestAdapterOptions, Surface, SurfaceConfiguration, SurfaceError, TextureUsages, Queue,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[allow(clippy::single_match)]
#[allow(clippy::collapsible_match)]
fn main() -> anyhow::Result<()> {
    // Logging
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

    // winit
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop)?;

    // wgpu
    let size = window.inner_size();
    let instance = wgpu::Instance::new(Backends::all());
    // SAFETY: Window handle is valid (except for Android) and will live shorter than window
    // In Android, we should first wait for "Resumed" event (see docs of `impl HasRawWindowHandle
    // for Window`)
    let surface = unsafe { instance.create_surface(&window) };
    // This is a handle to our actual graphics card
    let adapter = block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: Default::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .context("Failed to request an adapter")?;
    // Connects a physical device to create a logical device
    let (device, queue) = block_on(adapter.request_device(
        &DeviceDescriptor {
            label: None,
            features: wgpu::Features::empty(),
            limits: Default::default(),
        },
        None,
    ))
    .context("Failed to request a device")?;
    let mut config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: surface
            .get_preferred_format(&adapter)
            .context("Failed to get preferred format")?,
        width: size.width.max(1),
        height: size.height.max(1),
        present_mode: wgpu::PresentMode::Fifo,
    };
    surface.configure(&device, &config);

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            window_id, event, ..
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(size)
            | WindowEvent::ScaleFactorChanged {
                new_inner_size: &mut size,
                ..
            } => {
                config.width = size.width.max(1);
                config.height = size.height.max(1);
                surface.configure(&device, &config);
            }
            _ => {}
        },
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            // self.update();
            match render(&device, &queue, &surface) {
                Ok(_) => {}
                Err(SurfaceError::Lost) => surface.configure(&device, &config),
                Err(SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => error!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        _ => {}
    })
}

fn render(device: &Device, queue: &Queue, surface: &Surface) -> Result<(), wgpu::SurfaceError> {
    let output = surface.get_current_texture()?;
    let view = &output.texture.create_view(&Default::default());
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.begin_render_pass(&RenderPassDescriptor {
        label: None,
        depth_stencil_attachment: None,
        color_attachments: &[RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: LoadOp::Clear(wgpu::Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                }),
                store: true,
            },
        }],
    });
    queue.submit([encoder.finish()]);
    output.present();
    Ok(())
}
