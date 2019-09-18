use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::device::{Device, DeviceExtensions};
use vulkano::framebuffer::{
    Framebuffer, FramebufferAbstract, RenderPass, RenderPassAbstract, RenderPassDesc, Subpass,
};
use vulkano::image::SwapchainImage;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::vertex::{SingleBufferDefinition, VertexSource};
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::swapchain;
use vulkano::swapchain::{
    AcquireError, PresentMode, SurfaceTransform, Swapchain, SwapchainCreationError,
};
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};

use vulkano_win::VkSurfaceBuild;

use winit::{Event, EventsLoop, Window, WindowBuilder, WindowEvent};

use std::sync::Arc;

type MyPipeline<D> = Arc<
    GraphicsPipeline<
        SingleBufferDefinition<Vertex>,
        Box<dyn PipelineLayoutAbstract + Send + Sync>,
        Arc<RenderPass<D>>,
    >,
>;

pub struct MyVisual<D>
// where
// Layout: GraphicsPipelineAbstract + Send,
// RenderP: Send
{
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    pipeline: MyPipeline<D>,
}

impl<D> MyVisual<D>
// where
where
    D: RenderPassDesc + Send + Sync + 'static, // Layout: PipelineLayoutAbstract + Send + Sync ,
{
    pub fn new(
        device: Arc<Device>,
        // rpass: Subpass<Arc<dyn RenderPassAbstract+Sync+Send>>
        rpass: Subpass<Arc<RenderPass<D>>>,
    ) -> Self
// RenderP: RenderPassDesc + Send + Sync
    {
        // We now create a buffer that will store the shape of our triangle.
        let vertex_buffer = {
            let mut points: Vec<Vertex> = vec![];
            for t in -10000..10000 {
                let x = (t as f32) * 0.0001; // to seconds
                let y = (x * 3.14159 * 2.0 * 1.0).sin() + 0.03 * (x * 3.14159 * 2.0 * 200.0).sin();
                points.push(Vertex { position: [x, y] });
            }

            CpuAccessibleBuffer::from_iter(
                device.clone(),
                BufferUsage::all(),
                points.iter().cloned(),
            )
            .unwrap()
        };

        let vs = vs::Shader::load(device.clone()).unwrap();
        let fs = fs::Shader::load(device.clone()).unwrap();

        let pipeline = Arc::new(
            GraphicsPipeline::start()
                // We need to indicate the layout of the vertices.
                // The type `SingleBufferDefinition` actually contains a template parameter corresponding
                // to the type of each vertex. But in this code it is automatically inferred.
                .vertex_input_single_buffer()
                // A Vulkan shader can in theory contain multiple entry points, so we have to specify
                // which one. The `main` word of `main_entry_point` actually corresponds to the name of
                // the entry point.
                .vertex_shader(vs.main_entry_point(), ())
                // The content of the vertex buffer describes a list of triangles.
                // .triangle_list()
                // .lines()
                // .point_list()
                .line_strip()
                // Use a resizable viewport set to draw over the entire window
                .viewports_dynamic_scissors_irrelevant(1)
                // See `vertex_shader`.
                .fragment_shader(fs.main_entry_point(), ())
                // We have to indicate which subpass of which render pass this pipeline is going to be used
                // in. The pipeline will only be usable from this particular subpass.
                .render_pass(rpass)
                // Now that our builder is filled, we call `build()` to obtain an actual pipeline.
                .build(device.clone())
                .unwrap(),
        );

        MyVisual {
            vertex_buffer,
            pipeline,
        }
    }

    pub fn draw(
        &self,
        started_renderer: AutoCommandBufferBuilder,
        dynamic_state: &mut DynamicState,
    ) -> AutoCommandBufferBuilder {
        let in_progress_renderer = started_renderer
            .draw(
                self.pipeline.clone(),
                dynamic_state,
                self.vertex_buffer.clone(),
                (),
                (),
            )
            .unwrap();
        in_progress_renderer
    }

    // pub fn mk_pipelines<L, Gp, V>(device: Arc<Device>, rpass: Subpass<L>) -> Vec<Arc<GraphicsPipeline>>
    // where L: RenderPassAbstract,
    // Gp: GraphicsPipelineAbstract + VertexSource<V> + Send + Sync + 'static + Clone
    // {

    // }
}

/// Vertex type!
#[derive(Default, Debug, Clone)]
struct Vertex {
    position: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position);

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
#version 450

layout(location = 0) in vec2 position;

void main() {
gl_Position = vec4(position, 0.0, 1.0);
}"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
#version 450

layout(location = 0) out vec4 f_color;

void main() {
f_color = vec4(1.0, 0.0, 0.0, 1.0);
}
"
    }
}
