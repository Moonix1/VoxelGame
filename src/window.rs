use std::sync::Arc;

use winit::{
    window::{Window as WinitWindow, WindowAttributes},
    dpi::PhysicalSize,
    event_loop::ActiveEventLoop,
};

#[allow(unused)]
pub struct Window<'a> {
    pub title: &'a str,
    pub size: PhysicalSize<u32>,
    pub core_window: Arc<WinitWindow>,
}

impl<'a> Window<'a> {
    pub fn build(title: &'a str, size: PhysicalSize<u32>, event_loop: &ActiveEventLoop) -> Self {
        let window_attributes = WindowAttributes::default()
            .with_title(title)
            .with_inner_size(size);

        Self {
            title,
            size,
            core_window: Arc::new(event_loop.create_window(window_attributes).unwrap()),
        }
    }
}