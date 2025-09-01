use sokol::gfx as sg;
use glam::{Mat4, Vec2, Vec4};
use std::mem;

use crate::engine::Camera2D;

#[repr(C)]
struct Vertex {
    pos: [f32; 2],
    color: [f32; 4],
}

#[repr(C)]
struct Uniforms {
    mvp: [[f32; 4]; 4],
}

#[derive(Copy, Clone)]
pub struct Quad {
    pub position: Vec2,
    pub size: Vec2,
    pub color: Vec4,
}

impl Quad {
    pub fn new(x: f32, y: f32, width: f32, height: f32, color: Vec4) -> Self {
        Self {
            position: Vec2::new(x, y),
            size: Vec2::new(width, height),
            color,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Circle {
    pub center: Vec2,
    pub radius: f32,
    pub color: Vec4,
    pub segments: u32, // Number of triangles to approximate the circle
}

impl Circle {
    pub fn new(x: f32, y: f32, radius: f32, color: Vec4) -> Self {
        Self {
            center: Vec2::new(x, y),
            radius,
            color,
            segments: 32, // Default to 32 segments for smooth appearance
        }
    }

    pub fn with_segments(mut self, segments: u32) -> Self {
        self.segments = segments.max(3); // Minimum 3 segments for a triangle
        self
    }
}

pub struct Renderer {
    pipeline: sg::Pipeline,
    bind: sg::Bindings,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

/// Implementation for new, init, flush.
/// Handles the pipelien and shaders and all that good stuf
impl Renderer {
    pub fn new() -> Self {
        Self {
            pipeline: sg::Pipeline::default(),
            bind: sg::Bindings::default(),
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn init(&mut self) {
        
        let vs_source = "
cbuffer uniforms : register(b0) {
    float4x4 mvp;
};

struct vs_in {
    float2 position : POSITION;
    float4 color    : COLOR;
};
    
struct vs_out {
    float4 position : SV_Position;
    float4 color    : COLOR;
};
vs_out main(vs_in inp) {
    vs_out outp;
    outp.position = mul(mvp, float4(inp.position, 0.0, 1.0));
    outp.color = inp.color;
    return outp;
}
\0";

        let fs_source = "
struct ps_in {
    float4 position : SV_Position;
    float4 color : COLOR;
};

float4 main(ps_in inp) : SV_Target0 {
    return inp.color;
}
\0";
    
        let shader = sg::make_shader(&sg::ShaderDesc {
            vertex_func: sg::ShaderFunction {
                source: vs_source.as_ptr() as *const i8,
                ..Default::default()
            },
            fragment_func: sg::ShaderFunction {
                source: fs_source.as_ptr() as *const i8,
                ..Default::default()
            },
            attrs: [
                sg::ShaderVertexAttr {
                    hlsl_sem_name: "POSITION\0".as_ptr() as *const i8,
                    hlsl_sem_index: 0,
                    ..Default::default()
                },
                sg::ShaderVertexAttr {
                    hlsl_sem_name: "COLOR\0".as_ptr() as *const i8,
                    hlsl_sem_index: 0,
                    ..Default::default()
                },
                sg::ShaderVertexAttr::default(),
                sg::ShaderVertexAttr::default(),
                sg::ShaderVertexAttr::default(),
                sg::ShaderVertexAttr::default(),
                sg::ShaderVertexAttr::default(),
                sg::ShaderVertexAttr::default(),
                sg::ShaderVertexAttr::default(),
                sg::ShaderVertexAttr::default(),
                sg::ShaderVertexAttr::default(),
                sg::ShaderVertexAttr::default(),
                sg::ShaderVertexAttr::default(),
                sg::ShaderVertexAttr::default(),
                sg::ShaderVertexAttr::default(),
                sg::ShaderVertexAttr::default(),
            ],
            uniform_blocks: [
                sg::ShaderUniformBlock {
                    stage: sg::ShaderStage::Vertex,
                    size: mem::size_of::<Uniforms>() as u32,
                    hlsl_register_b_n: 0, // matches "register(b0)" in shader
                    ..Default::default()
                },
                sg::ShaderUniformBlock::default(),
                sg::ShaderUniformBlock::default(),
                sg::ShaderUniformBlock::default(),
                sg::ShaderUniformBlock::default(),
                sg::ShaderUniformBlock::default(),
                sg::ShaderUniformBlock::default(),
                sg::ShaderUniformBlock::default(),
            ],
            ..Default::default()
        });

        // Create pipeline
        self.pipeline = sg::make_pipeline(&sg::PipelineDesc {
            shader,
            layout: sg::VertexLayoutState {
                attrs: [
                    sg::VertexAttrState {
                        buffer_index: 0,
                        offset: 0,
                        format: sg::VertexFormat::Float2,
                    }, // position
                    sg::VertexAttrState {
                        buffer_index: 0,
                        offset: 8, // 2 floats * 4 bytes = 8 bytes offset
                        format: sg::VertexFormat::Float4,
                    }, // color
                    sg::VertexAttrState::default(),
                    sg::VertexAttrState::default(),
                    sg::VertexAttrState::default(),
                    sg::VertexAttrState::default(),
                    sg::VertexAttrState::default(),
                    sg::VertexAttrState::default(),
                    sg::VertexAttrState::default(),
                    sg::VertexAttrState::default(),
                    sg::VertexAttrState::default(),
                    sg::VertexAttrState::default(),
                    sg::VertexAttrState::default(),
                    sg::VertexAttrState::default(),
                    sg::VertexAttrState::default(),
                    sg::VertexAttrState::default(),
                ],
                buffers: [
                    sg::VertexBufferLayoutState {
                        stride: mem::size_of::<Vertex>() as i32,
                        step_func: sg::VertexStep::PerVertex,
                        step_rate: 1,
                    },
                    sg::VertexBufferLayoutState::default(),
                    sg::VertexBufferLayoutState::default(),
                    sg::VertexBufferLayoutState::default(),
                    sg::VertexBufferLayoutState::default(),
                    sg::VertexBufferLayoutState::default(),
                    sg::VertexBufferLayoutState::default(),
                    sg::VertexBufferLayoutState::default(),
                ],
            },
            index_type: sg::IndexType::Uint16,
            ..Default::default()
        });

        // Create vertex buffer
        let vbuf = sg::make_buffer(&sg::BufferDesc {
            size: (1000 * mem::size_of::<Vertex>()), // Space for many vertices
            usage: sg::BufferUsage {
                vertex_buffer: true,
                stream_update: true,
                ..Default::default()
            },
            ..Default::default()
        });

        // Create index buffer
        let ibuf = sg::make_buffer(&sg::BufferDesc {
            size: (1500 * mem::size_of::<u16>()),
            usage: sg::BufferUsage {
                index_buffer: true,
                stream_update: true,
                ..Default::default()
            },
            ..Default::default()
        });

        self.bind.vertex_buffers[0] = vbuf;
        self.bind.index_buffer = ibuf;

        println!("Renderer initialized with shaders and buffers");

    }

    pub fn flush(&mut self, camera: &mut Camera2D) {
        if self.vertices.is_empty() {
            return;
        }

        // Update vertex buffer
        sg::update_buffer(
            self.bind.vertex_buffers[0],
            &sg::Range {
                ptr: self.vertices.as_ptr() as *const _,
                size: self.vertices.len() * mem::size_of::<Vertex>(),
            },
        );

        // Update index buffer
        sg::update_buffer(
            self.bind.index_buffer,
            &sg::Range {
                ptr: self.indices.as_ptr() as *const _,
                size: self.indices.len() * mem::size_of::<u16>(),
            },
        );

        // Set up orthographic projection matrix
        // let width = sokol::app::width() as f32;
        // let height = sokol::app::height() as f32;
        // let ortho = Mat4::orthographic_rh(0.0, width, height, 0.0, -1.0, 1.0);
        
        // Use camera's view-projection matrix instead of hardcoded orthographic
        let view_proj = camera.get_view_projection_matrix();

        let uniforms = Uniforms {
            mvp: view_proj.to_cols_array_2d(),
        };

        // Render
        sg::apply_pipeline(self.pipeline);
        sg::apply_bindings(&self.bind);
        sg::apply_uniforms(0, &sg::Range {
            ptr: &uniforms as *const _ as *const _,
            size: mem::size_of::<Uniforms>(),
        });

        sg::draw(0, self.indices.len(), 1);

        // Clear for next frame
        self.vertices.clear();
        self.indices.clear();
    }
}

/// Implementation for drawing to the screen used by the game
impl Renderer {
    pub fn draw_quad(&mut self, quad: &Quad) {
        let start_vertex = self.vertices.len() as u16;
        
        // Create 4 vertices for the quad
        let x1 = quad.position.x;
        let y1 = quad.position.y;
        let x2 = quad.position.x + quad.size.x;
        let y2 = quad.position.y + quad.size.y;
        
        let color = [quad.color.x, quad.color.y, quad.color.z, quad.color.w];

        // Add vertices (clockwise)
        self.vertices.push(Vertex { pos: [x1, y1], color });
        self.vertices.push(Vertex { pos: [x2, y1], color });
        self.vertices.push(Vertex { pos: [x2, y2], color });
        self.vertices.push(Vertex { pos: [x1, y2], color });
        
        // Add indices for two triangles
        let indices = [
            start_vertex, start_vertex + 1, start_vertex + 2,
            start_vertex, start_vertex + 2, start_vertex + 3,
        ];
        
        self.indices.extend_from_slice(&indices);
    }

    pub fn draw_circle(&mut self, circle: &Circle) {
        let center_vertex = self.vertices.len() as u16;
        let color = [circle.color.x, circle.color.y, circle.color.z, circle.color.w];
        
        // Add center vertex
        self.vertices.push(Vertex { 
            pos: [circle.center.x, circle.center.y], 
            color 
        });
        
        // Add vertices around the circumference
        for i in 0..circle.segments {
            let angle = (i as f32 / circle.segments as f32) * 2.0 * std::f32::consts::PI;
            let x = circle.center.x + angle.cos() * circle.radius;
            let y = circle.center.y + angle.sin() * circle.radius;
            
            self.vertices.push(Vertex { pos: [x, y], color });
        }
        
        // Add triangles from center to each edge
        for i in 0..circle.segments {
            let next = (i + 1) % circle.segments;
            self.indices.extend_from_slice(&[
                center_vertex,                    // center
                center_vertex + 1 + i as u16,    // current point on circumference
                center_vertex + 1 + next as u16, // next point on circumference
            ]);
        }
    }

}