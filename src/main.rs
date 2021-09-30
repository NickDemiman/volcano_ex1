use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState, SubpassContents}; 
use vulkano::device::{Device, DeviceExtensions, physical::PhysicalDevice}; 
use vulkano::render_pass::{Framebuffer, FramebufferAbstract, RenderPass}; 
use vulkano::image::SwapchainImage; 
use vulkano::instance::{Instance}; 
use vulkano::pipeline::viewport::Viewport; 
use vulkano::swapchain::{AcquireError, PresentMode, SurfaceTransform, Swapchain, SwapchainCreationError}; 
use vulkano::swapchain; 
use vulkano::sync::{GpuFuture, FlushError}; 
use vulkano::sync; 
use vulkano_win::VkSurfaceBuild; 
use winit::window::{WindowBuilder, Window}; 
use winit::event_loop::{EventLoop, ControlFlow, EventLoopWindowTarget}; 
use winit::event::{Event, WindowEvent}; 
use std::sync::Arc; 



fn main() 
{
    // Создаем инстанс
    let _instance = 
    { 
        let extensions = vulkano_win::required_extensions();
        Instance::new(None, vulkano::Version::V1_1, &extensions, None).unwrap() 
    };

    // Определяем устройство, где будем выполнять рендер
    let physical_device = PhysicalDevice::enumerate(&_instance).next().unwrap();

    // Создаем окно
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new();

    let surface: Arc<swapchain::Surface<Window>> = VkSurfaceBuild::build_vk_surface(window, &event_loop, _instance.clone()).unwrap();

    // Создадим queue_family для дальнейшего создания экземпляров Queue
    let queue_family = physical_device
                                    .queue_families()
                                    .find(|&q| { 
                                                  q.supports_graphics() && surface.is_supported(q).unwrap_or(false) 
                                                }).unwrap();

    // Создаем экземпляр Device для отправки привязки нашей очереди к ГПУ
    let device_ext = DeviceExtensions { khr_swapchain: true, .. DeviceExtensions::none() }; 
    let (device, mut queues) = Device::new(
                                            physical_device, 
                                            physical_device.supported_features(), 
                                            &device_ext, 
                                            [(queue_family, 0.5)].iter().cloned()
                                           ).unwrap();
    
    // Создадим очередь
    let queue = queues.next().unwrap();

    // Создадим свапчейн
    let (mut swapchain, images) = 
    { 
        let caps = surface.capabilities(physical_device).unwrap(); 
        let usage = caps.supported_usage_flags; 
        let alpha = caps.supported_composite_alpha.iter().next().unwrap(); 
        let format = caps.supported_formats[0].0; 
        let dimensions: [u32; 2] = surface.window().inner_size().into();

        Swapchain::start(device.clone(), surface.clone())
                                                        .num_images(caps.min_image_count)
                                                        .format(format)
                                                        .dimensions(dimensions)
                                                        .layers(1)
                                                        .usage(usage)
                                                        .transform(SurfaceTransform::Identity)
                                                        .composite_alpha(alpha)
                                                        .present_mode(PresentMode::Fifo)
                                                        .fullscreen_exclusive(swapchain::FullscreenExclusive::Default)
                                                        .clipped(true)
                                                        .color_space(swapchain::ColorSpace::SrgbNonLinear)
                                                        .build()
                                                        .unwrap()
    
    };
    
    // Создадим массив вершин
    #[derive(Default, Debug, Clone)] struct Vertex
    {
        position: [f32; 3],
        col:[f32; 4]
    } 
    vulkano::impl_vertex!(Vertex, position, col); 
    
    // Создание шейдера для отрисовки треугольника
    mod vs
    {
        vulkano_shaders::shader!
        {
            ty: "vertex", 
            src: " #version 450
                layout(location = 0) in vec3 position; 
                void main() { gl_Position = vec4(position, 1.0); }" 
        }
    } 
    
    mod fs 
    { 
        vulkano_shaders::shader!
        {
            ty: "fragment", 
            src: " #version 450
                layout(location = 0) out vec4 f_color;
                void main() {
                f_color = vec4(1.0, 0.0, 0.0, 1.0); } " 
        } 
    } 
    
    let vs = vs::Shader::load(device.clone()).unwrap();
    let fs = fs::Shader::load(device.clone()).unwrap();
       
}
