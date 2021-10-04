use vulkano::buffer::{cpu_access::CpuAccessibleBuffer, BufferUsage};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState, SubpassContents, CommandBufferUsage}; 
use vulkano::device::{Device, DeviceExtensions, physical::PhysicalDevice}; 
use vulkano::render_pass::{Framebuffer, FramebufferAbstract, RenderPass, Subpass}; 
use vulkano::image::{SwapchainImage, view::ImageView}; 
use vulkano::instance::Instance; 
use vulkano::pipeline::{viewport::Viewport, GraphicsPipeline}; 
use vulkano::swapchain; 
use vulkano::swapchain::{AcquireError, PresentMode, SurfaceTransform, Swapchain, SwapchainCreationError}; 
use vulkano::sync; 
use vulkano::sync::{GpuFuture, FlushError}; 
use vulkano_win::VkSurfaceBuild; 
use winit::event::{Event, WindowEvent}; 
use winit::event_loop::{EventLoop, ControlFlow}; 
use winit::window::{WindowBuilder, Window}; 
use std::sync::Arc; 
use std::option::Option;

fn InitWindowAndChooseDevice()
{
    // Создаем инстанс
    let _instance = { 
        let extensions = vulkano_win::required_extensions();
        Instance::new(None, vulkano::Version::V1_1, &extensions, None).unwrap() 
    };

    // Определяем устройство, где будем выполнять рендер
    let physical_device = PhysicalDevice::enumerate(&_instance).next().unwrap();

    // Создаем окно
    let event_loop = EventLoop::new();
    let surface: Arc<swapchain::Surface<Window>> = VkSurfaceBuild::build_vk_surface(
        WindowBuilder::new(), 
        &event_loop, 
        _instance.clone()
    ).unwrap();

    // Создадим queue_family для дальнейшего создания экземпляров Queue
    let queue_family = physical_device.queue_families().find(|&q| { 
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

    (surface, )
}

fn main() 
{
    InitWindowAndChooseDevice();

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

    // Создадим экземпляр структуры Renderpass, для настройки конвеера рендера
    let render_pass = Arc::new(vulkano::single_pass_renderpass!
        (   
            device.clone(),  
            attachments:
            {  
                color: 
                {
                    load: Clear,  
                    store: Store,  
                    format: swapchain.format(),  
                    samples: 1, 
                }  
            },  
            pass: 
            {  
                color: [color],  
                depth_stencil: {}  
            } 
        ).unwrap()
    );
    
    // Создадим пайплайн
    let mut dynamic_state = DynamicState 
    { 
        line_width: None, 
        viewports: None,
        scissors: None, 
        compare_mask: None, 
        write_mask: None, 
        reference: None 
    }; 
    
    let pipeline = Arc::new(
        GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex>()
            .vertex_shader(vs.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .render_pass(
                Subpass::from(render_pass.clone(), 0)
                    .unwrap()
            )
            .build(device.clone())
            .unwrap()
    );

    // Создание буффера
    let vertex_buffer = CpuAccessibleBuffer::from_iter
    (
        device.clone(), 
        BufferUsage::all(), 
        false, 
        [ 
            Vertex 
            {
                position: [-0.5, 0.5, 0.0],
                col : [0.0,0.0,0.0,0.0]
            },

            Vertex
            {
                position: [0.5, 0.5, 0.0],
                col : [0.0,0.0,0.0,0.0]
            }, 

            Vertex
            {
                position: [0.0, -0.5, 0.0],
                col : [0.0,0.0,0.0,0.0]
            }
        ].iter().cloned()
    ).unwrap();

    let mut framebuffers = window_size_dependent_setup(&images, render_pass.clone(), &mut dynamic_state);

    let mut recreate_swapchain = false;

    let mut previous_frame_end = Some( 
        Box::new(
            sync::now(device.clone())
        ) as Box<dyn GpuFuture>
    );

    event_loop.run(move |event, _, control_flow| 
        {
            match event
            { 
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => { 
                    *control_flow = ControlFlow::Exit; 
                },
                
                Event::WindowEvent { event: WindowEvent::Resized(_), .. } => { 
                    recreate_swapchain = true; 
                },

                Event::RedrawEventsCleared => 
                { 
                    previous_frame_end.as_mut().take().unwrap().cleanup_finished();

                    if recreate_swapchain 
                    { 
                        let dimensions: [u32; 2] = surface.window().inner_size().into();
                        let (new_swapchain, new_images) = match swapchain.recreate().dimensions(dimensions).build()
                        { 
                            Ok(r) => r, 
                            Err(SwapchainCreationError::UnsupportedDimensions) => return, 
                            Err(e) => panic!("Failed to recreate swapchain: {:?}", e) 
                        };

                        swapchain = new_swapchain; 
                        framebuffers = window_size_dependent_setup(&new_images, render_pass.clone(), &mut dynamic_state); 
                        recreate_swapchain = false; 
                    }
                    
                    let (image_num, suboptimal, acquire_future) = match swapchain::acquire_next_image(swapchain.clone(), None)
                    { 
                        Ok(r) => r, 
                        Err(AcquireError::OutOfDate) => 
                        { 
                            recreate_swapchain = true;
                            return; 
                        },
                        Err(e) => panic!("Failed to acquire next image: {:?}", e) 
                    };

                    if suboptimal 
                    { 
                        recreate_swapchain = true; 
                    }

                    let clear_values = vec!([0.0, 0.0, 0.0, 1.0].into());

                    let mut cmd_buffer_builder = AutoCommandBufferBuilder::primary(
                        device.clone(), 
                        queue.family(), 
                        CommandBufferUsage::OneTimeSubmit
                    ).unwrap();
                    cmd_buffer_builder
                        .begin_render_pass(framebuffers[image_num].clone(), SubpassContents::Inline, clear_values)
                        .unwrap()
                        .draw(pipeline.clone(), &dynamic_state, vertex_buffer.clone(), (), ())
                        .unwrap()
                        .end_render_pass()
                    .unwrap();

                    let command_buffer = cmd_buffer_builder.build().unwrap();

                    let future = previous_frame_end.take().unwrap()
                        .join(acquire_future) 
                        .then_execute(queue.clone(), command_buffer).unwrap()
                        .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
                    .then_signal_fence_and_flush();

                    match future {
                        Ok(future) => { 
                            previous_frame_end = Some(Box::new(future) as Box<_>); 
                        }

                        Err(FlushError::OutOfDate) => { 
                            recreate_swapchain = true; previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<_>); 
                        }

                        Err(e) => {
                            println!("Failed to flush future: {:?}", e);
                            previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<_>); 
                        } 
                    }

                }, 
                
                _ => {} 
            }

        }); // event_loop end

}

fn window_size_dependent_setup( 
    images: &[Arc<SwapchainImage<Window>>], 
    render_pass: Arc<RenderPass>, 
    dynamic_state: &mut DynamicState 
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> 
{ 
    let dimensions = images[0].dimensions(); 
    let viewport = Viewport
    {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32,dimensions[1] as f32],
        depth_range: 0.0 .. 1.0,
    };

    dynamic_state.viewports = Some(vec!(viewport)); 
    images.iter().map(|image|
        { 
            let image_view = ImageView::new(image.clone()).unwrap();
            Arc::new( 
                    Framebuffer::start(render_pass.clone())
                        .add(image_view)
                        .unwrap()
                        .build()
                        .unwrap()
                ) as Arc<dyn FramebufferAbstract + Send + Sync> 
        }).collect::<Vec<_>>()
}