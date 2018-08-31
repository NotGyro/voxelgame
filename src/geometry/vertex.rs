#[derive(Debug, Clone)]
pub struct VertexPositionNormalUVColor {
    pub position: [f32; 3],
    pub normal:   [f32; 3],
    pub uv:       [f32; 2],
    pub color:    [f32; 3]
}
impl_vertex!(VertexPositionNormalUVColor, position, normal, uv, color);


#[derive(Debug, Clone)]
pub struct VertexPositionColorAlpha {
    pub position: [f32; 3],
    pub color:    [f32; 4]
}
impl_vertex!(VertexPositionColorAlpha, position, color);


#[derive(Debug, Clone)]
pub struct VertexPosition {
    pub position: [f32; 3]
}
impl_vertex!(VertexPosition, position);


#[derive(Debug, Clone)]
pub struct VertexPositionUV {
    pub position: [f32; 3],
    pub uv:       [f32; 2]
}
impl_vertex!(VertexPositionUV, position, uv);