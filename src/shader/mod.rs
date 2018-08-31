pub mod chunks {
    #[allow(dead_code)]
    pub mod vertex {
        #[derive(VulkanoShader)]
        #[ty = "vertex"]
        #[path = "src/shader/chunks.vert"]
        struct Dummy;
    }

    #[allow(dead_code)]
    pub mod fragment {
        #[derive(VulkanoShader)]
        #[ty = "fragment"]
        #[path = "src/shader/chunks.frag"]
        struct Dummy;
    }
}

pub mod lines {
    #[allow(dead_code)]
    pub mod vertex {
        #[derive(VulkanoShader)]
        #[ty = "vertex"]
        #[path = "src/shader/lines.vert"]
        struct Dummy;
    }

    #[allow(dead_code)]
    pub mod fragment {
        #[derive(VulkanoShader)]
        #[ty = "fragment"]
        #[path = "src/shader/lines.frag"]
        struct Dummy;
    }
}

pub mod skybox {
    #[allow(dead_code)]
    pub mod vertex {
        #[derive(VulkanoShader)]
        #[ty = "vertex"]
        #[path = "src/shader/skybox.vert"]
        struct Dummy;
    }

    #[allow(dead_code)]
    pub mod fragment {
        #[derive(VulkanoShader)]
        #[ty = "fragment"]
        #[path = "src/shader/skybox.frag"]
        struct Dummy;
    }
}
