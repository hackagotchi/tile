use iced_wgpu::{wgpu, Viewport};
use iced_winit::winit;
use winit::window::Window;
use winit::dpi::PhysicalSize;

pub struct RenderingState {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub swap_chain: wgpu::SwapChain,
    pub swap_chain_descriptor: wgpu::SwapChainDescriptor,
    pub viewport: Viewport,
}

impl RenderingState {
    pub fn new(window: &Window) -> Self {
        // Initialize wgpu
        let surface = wgpu::Surface::create(window);
        let (device, queue) = futures::executor::block_on(async {
            let adapter = wgpu::Adapter::request(
                &wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                },
                wgpu::BackendBit::PRIMARY,
            )
            .await
            .expect("Request adapter");

            adapter
                .request_device(&wgpu::DeviceDescriptor {
                    extensions: wgpu::Extensions {
                        anisotropic_filtering: false,
                    },
                    limits: wgpu::Limits::default(),
                })
                .await
        });

        let (swap_chain, swap_chain_descriptor) = {
            let size = window.inner_size();
            let sc_desc = wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::Fifo,
            };

            (device.create_swap_chain(&surface, &sc_desc), sc_desc)
        };

        let physical_size = window.inner_size();
        let viewport = Viewport::with_physical_size(
            iced_winit::Size::new(physical_size.width, physical_size.height),
            window.scale_factor(),
        );

        Self {
            surface,
            device,
            queue,
            swap_chain,
            swap_chain_descriptor,
            viewport,
        }
    }

    pub fn resize(&mut self, screen: PhysicalSize<u32>, window: &Window) {
        self.viewport = Viewport::with_physical_size(
            iced_winit::Size::new(screen.width, screen.height),
            window.scale_factor()
        );
        self.swap_chain_descriptor.width = screen.width;
        self.swap_chain_descriptor.height = screen.height;
        self.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_descriptor);
    }
}
