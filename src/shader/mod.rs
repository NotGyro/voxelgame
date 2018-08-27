pub mod default {
    #[allow(dead_code)]
    pub mod vertex {
        #[derive(VulkanoShader)]
        #[ty = "vertex"]
        #[path = "src/shader/default.vert"]
        struct Dummy;
    }

    #[allow(dead_code)]
    pub mod fragment {
        #[derive(VulkanoShader)]
        #[ty = "fragment"]
        #[path = "src/shader/default.frag"]
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
