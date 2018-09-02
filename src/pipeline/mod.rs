//! Rendering pipeline types.


pub mod chunk_pipeline;
pub mod lines_pipeline;
pub mod skybox_pipeline;
pub use self::chunk_pipeline::ChunkRenderPipeline;
pub use self::lines_pipeline::LinesRenderPipeline;
pub use self::skybox_pipeline::SkyboxRenderPipeline;


// TODO: make render pipelines generic
pub trait RenderPipelineAbstract {
}