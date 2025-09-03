use sokol::gfx as sg;
use glam::{Vec2, Vec4};
use std::{collections::HashMap, mem, string};

use crate::engine::{texture, AnimationState, Camera2D, Particle, TextureManager};

#[repr(C)]
struct Vertex {
    pos: [f32; 2],
    texcoord: [f32; 2],
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

#[derive(Clone)]
pub struct Sprite {
    pub position: Vec2,
    pub size: Vec2,
    pub uv: Vec4,                  
    pub color: Vec4,               
    pub rotation: f32,
    pub texture: Option<sg::Image>,
    pub texture_name: String,
    pub animation_state: Option<AnimationState>,
    pub flip_x: bool,
    pub flip_y: bool, 
}

impl Sprite {
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            size: Vec2::new(32.0, 32.0),
            uv: Vec4::new(0.0, 0.0, 1.0, 1.0),
            color: Vec4::ONE,
            rotation: 0.0,
            texture: None,
            texture_name: String::new(),
            animation_state: None,
            flip_x: false,
            flip_y: false,
        }
    }

    pub fn with_texture(mut self, texture_name: String, texture: sg::Image) -> Self {
        self.texture = Some(texture);
        self.texture_name = texture_name;
        self
    }

    pub fn with_texture_name(mut self, texture_name: String) -> Self {
        self.texture_name = texture_name;
        self
    }

    pub fn with_position(mut self, pos: Vec2) -> Self {
        self.position = pos;
        self
    }

    pub fn with_size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    pub fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    pub fn with_uv(mut self, uv: Vec4) -> Self {
        self.uv = uv;
        self
    }

    pub fn with_flip_x(mut self, flip: bool) -> Self {
        self.flip_x = flip;
        self
    }

    pub fn with_flip_y(mut self, flip: bool) -> Self {
        self.flip_y = flip;
        self
    }

    pub fn change_texture(&mut self, texture_name: String) {
        self.texture_name = texture_name;
    }
}

struct DrawBatch {
    texture: sg::Image,
    start_index: usize,
    index_count: usize,
}

pub struct Renderer {
    textured_pipeline: sg::Pipeline,
    colored_pipeline: sg::Pipeline,
    bind: sg::Bindings,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    texture_manager: TextureManager,
    batches: Vec<DrawBatch>,
    sampler: sg::Sampler,
    vbuf_size: usize,
    ibuf_size: usize,
    view_cache: HashMap<u32, sg::View>,
}

/// Implementation for new, init, flush.
/// Handles the pipelien and shaders and all that good stuf
impl Renderer {
    pub fn new() -> Self {
        Self {
            textured_pipeline: sg::Pipeline::default(),
            colored_pipeline: sg::Pipeline::default(),
            bind: sg::Bindings::default(),
            vertices: Vec::new(),
            indices: Vec::new(),
            texture_manager: TextureManager::new(),
            batches: Vec::new(),
            sampler: sg::Sampler::default(),
            vbuf_size: 0,
            ibuf_size: 0,
            view_cache: HashMap::new(),
        }
    }

    pub fn init(&mut self) {
        self.texture_manager.init();

        // Create sampler for texture filtering
        self.sampler = sg::make_sampler(&sg::SamplerDesc {
            min_filter: sg::Filter::Nearest,  // or Nearest for pixel art
            mag_filter: sg::Filter::Nearest,  // or Nearest for pixel art
            wrap_u: sg::Wrap::ClampToEdge,
            wrap_v: sg::Wrap::ClampToEdge,
            ..Default::default()
        });

        let textured_vs_source = "
cbuffer uniforms : register(b0) {
    float4x4 mvp;
};

struct vs_in {
    float2 position : POSITION;
    float2 texcoord : TEXCOORD;
    float4 color    : COLOR;
};
    
struct vs_out {
    float4 position : SV_Position;
    float2 texcoord : TEXCOORD;
    float4 color    : COLOR;
};

vs_out main(vs_in inp) {
    vs_out outp;
    outp.position = mul(mvp, float4(inp.position, 0.0, 1.0));
    outp.texcoord = inp.texcoord;
    outp.color = inp.color;
    return outp;
}
\0";

        let textured_fs_source = "
Texture2D tex : register(t0);
SamplerState smp : register(s0);

struct ps_in {
    float4 position : SV_Position;
    float2 texcoord : TEXCOORD;
    float4 color : COLOR;
};

float4 main(ps_in inp) : SV_Target0 {
    float4 tex_color = tex.Sample(smp, inp.texcoord);
    return tex_color * inp.color;
}
\0";

        let color_vs_source = "
cbuffer uniforms : register(b0) {
    float4x4 mvp;
};

struct vs_in {
    float2 position : POSITION;
    float2 texcoord : TEXCOORD;
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

        let color_fs_source = "
Texture2D tex : register(t0);
SamplerState smp : register(s0);

struct ps_in {
    float4 position : SV_Position;
    float4 color : COLOR;
};

float4 main(ps_in inp) : SV_Target0 {
    return inp.color;
}
\0";
        
        let texture_shader = sg::make_shader(&sg::ShaderDesc {
            vertex_func: sg::ShaderFunction {
                source: textured_vs_source.as_ptr() as *const i8,
                ..Default::default()
            },
            fragment_func: sg::ShaderFunction {
                source: textured_fs_source.as_ptr() as *const i8,
                ..Default::default()
            },
            attrs: [
                sg::ShaderVertexAttr {
                    hlsl_sem_name: "POSITION\0".as_ptr() as *const i8,
                    hlsl_sem_index: 0,
                    ..Default::default()
                },
                sg::ShaderVertexAttr {
                    hlsl_sem_name: "TEXCOORD\0".as_ptr() as *const i8,
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
            // Define the texture view (image resource)
            views: [
                sg::ShaderView {
                    texture: sg::ShaderTextureView {
                        stage: sg::ShaderStage::Fragment,
                        image_type: sg::ImageType::Dim2,  // 2D texture
                        sample_type: sg::ImageSampleType::Float,
                        multisampled: false,
                        hlsl_register_t_n: 0,  // Maps to register(t0) in HLSL
                        msl_texture_n: 0,
                        wgsl_group1_binding_n: 0,
                    },
                    storage_buffer: sg::ShaderStorageBufferView::default(),
                    storage_image: sg::ShaderStorageImageView::default(),
                },
                // Fill remaining with defaults (28 total)
                sg::ShaderView::default(),
                sg::ShaderView::default(),
                sg::ShaderView::default(),
                sg::ShaderView::default(),
                sg::ShaderView::default(),
                sg::ShaderView::default(),
                sg::ShaderView::default(),
                sg::ShaderView::default(),
                sg::ShaderView::default(),
                sg::ShaderView::default(),
                sg::ShaderView::default(),
                sg::ShaderView::default(),
                sg::ShaderView::default(),
                sg::ShaderView::default(),
                sg::ShaderView::default(),
                sg::ShaderView::default(),
                sg::ShaderView::default(),
                sg::ShaderView::default(),
                sg::ShaderView::default(),
                sg::ShaderView::default(),
                sg::ShaderView::default(),
                sg::ShaderView::default(),
                sg::ShaderView::default(),
                sg::ShaderView::default(),
                sg::ShaderView::default(),
                sg::ShaderView::default(),
                sg::ShaderView::default(),
            ],
            // Define the sampler
            samplers: [
                sg::ShaderSampler {
                    stage: sg::ShaderStage::Fragment,
                    sampler_type: sg::SamplerType::Filtering,
                    hlsl_register_s_n: 0,  // Maps to register(s0) in HLSL
                    ..Default::default()
                },
                // Fill remaining with defaults (16 total)
                sg::ShaderSampler::default(),
                sg::ShaderSampler::default(),
                sg::ShaderSampler::default(),
                sg::ShaderSampler::default(),
                sg::ShaderSampler::default(),
                sg::ShaderSampler::default(),
                sg::ShaderSampler::default(),
                sg::ShaderSampler::default(),
                sg::ShaderSampler::default(),
                sg::ShaderSampler::default(),
                sg::ShaderSampler::default(),
                sg::ShaderSampler::default(),
                sg::ShaderSampler::default(),
                sg::ShaderSampler::default(),
                sg::ShaderSampler::default(),
            ],
            // Link texture view and sampler together
            texture_sampler_pairs: [
                sg::ShaderTextureSamplerPair {
                    stage: sg::ShaderStage::Fragment,
                    view_slot: 0,
                    sampler_slot: 0,
                    glsl_name: std::ptr::null()
                },
                sg::ShaderTextureSamplerPair::default(),
                sg::ShaderTextureSamplerPair::default(),
                sg::ShaderTextureSamplerPair::default(),
                sg::ShaderTextureSamplerPair::default(),
                sg::ShaderTextureSamplerPair::default(),
                sg::ShaderTextureSamplerPair::default(),
                sg::ShaderTextureSamplerPair::default(),
                sg::ShaderTextureSamplerPair::default(),
                sg::ShaderTextureSamplerPair::default(),
                sg::ShaderTextureSamplerPair::default(),
                sg::ShaderTextureSamplerPair::default(),
                sg::ShaderTextureSamplerPair::default(),
                sg::ShaderTextureSamplerPair::default(),
                sg::ShaderTextureSamplerPair::default(),
                sg::ShaderTextureSamplerPair::default()
            ],
            ..Default::default()
        });

        let colored_shader = sg::make_shader(&sg::ShaderDesc {
            vertex_func: sg::ShaderFunction {
                source: color_vs_source.as_ptr() as *const i8,
                ..Default::default()
            },
            fragment_func: sg::ShaderFunction {
                source: color_fs_source.as_ptr() as *const i8,
                ..Default::default()
            },
            attrs: [
                sg::ShaderVertexAttr {
                    hlsl_sem_name: "POSITION\0".as_ptr() as *const i8,
                    hlsl_sem_index: 0,
                    ..Default::default()
                },
                sg::ShaderVertexAttr {
                    hlsl_sem_name: b"TEXCOORD\0".as_ptr() as *const i8,
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
            ],
            uniform_blocks: [
                sg::ShaderUniformBlock {
                    stage: sg::ShaderStage::Vertex,
                    size: mem::size_of::<Uniforms>() as u32,
                    hlsl_register_b_n: 0,
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

        // Vertex layout (same for both pipelines)
        let vertex_layout = sg::VertexLayoutState {
            attrs: [
                sg::VertexAttrState {
                    buffer_index: 0,
                    offset: 0,
                    format: sg::VertexFormat::Float2,
                }, // position
                sg::VertexAttrState {
                    buffer_index: 0,
                    offset: 8,
                    format: sg::VertexFormat::Float2,
                }, // texcoord
                sg::VertexAttrState {
                    buffer_index: 0,
                    offset: 16,
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
        };

        // Create pipeline
        self.textured_pipeline = sg::make_pipeline(&sg::PipelineDesc {
            shader: texture_shader,
            layout: vertex_layout,
            index_type: sg::IndexType::Uint16,
            primitive_type: sg::PrimitiveType::Triangles,
            cull_mode: sg::CullMode::None,
            depth: sg::DepthState {
                write_enabled: false,
                compare: sg::CompareFunc::Always,
                ..Default::default()
            },
            colors: [
                sg::ColorTargetState {
                    blend: sg::BlendState {
                        enabled: true,
                        src_factor_rgb: sg::BlendFactor::SrcAlpha,
                        dst_factor_rgb: sg::BlendFactor::OneMinusSrcAlpha,
                        src_factor_alpha: sg::BlendFactor::One,
                        dst_factor_alpha: sg::BlendFactor::OneMinusSrcAlpha,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                sg::ColorTargetState::default(),
                sg::ColorTargetState::default(),
                sg::ColorTargetState::default(),
            ],
            ..Default::default()
        });

        self.colored_pipeline = sg::make_pipeline(&sg::PipelineDesc {
            shader: colored_shader,
            layout: vertex_layout,
            index_type: sg::IndexType::Uint16,
            primitive_type: sg::PrimitiveType::Triangles,
            cull_mode: sg::CullMode::None,
            depth: sg::DepthState {
                write_enabled: false,
                compare: sg::CompareFunc::Always,
                ..Default::default()
            },
            colors: [
                sg::ColorTargetState {
                    blend: sg::BlendState {
                        enabled: true,
                        src_factor_rgb: sg::BlendFactor::SrcAlpha,
                        dst_factor_rgb: sg::BlendFactor::OneMinusSrcAlpha,
                        src_factor_alpha: sg::BlendFactor::One,
                        dst_factor_alpha: sg::BlendFactor::OneMinusSrcAlpha,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                sg::ColorTargetState::default(),
                sg::ColorTargetState::default(),
                sg::ColorTargetState::default(),
            ],
            ..Default::default()
        });

        let initial_vtx_count = 1000usize;
        let initial_idx_count = 1500usize;

        let vbuf_size_bytes = initial_vtx_count * mem::size_of::<Vertex>();
        let ibuf_size_bytes = initial_idx_count * mem::size_of::<u16>();

        // Create vertex buffer
        let vbuf = sg::make_buffer(&sg::BufferDesc {
            size: vbuf_size_bytes, // Space for many vertices
            usage: sg::BufferUsage {
                vertex_buffer: true,
                stream_update: true,
                ..Default::default()
            },
            ..Default::default()
        });

        // Create index buffer
        let ibuf = sg::make_buffer(&sg::BufferDesc {
            size: ibuf_size_bytes,
            usage: sg::BufferUsage {
                index_buffer: true,
                stream_update: true,
                ..Default::default()
            },
            ..Default::default()
        });

        self.bind.vertex_buffers[0] = vbuf;
        self.bind.index_buffer = ibuf;
        self.vbuf_size = vbuf_size_bytes;
        self.ibuf_size = ibuf_size_bytes;
        self.bind.samplers[0] = self.sampler;

        println!("Renderer initialized with shaders and buffers");

    }

    pub fn begin_frame(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.batches.clear();
    }

    pub fn flush(&mut self, camera: &mut Camera2D) {
        if self.vertices.is_empty() {
            return;
        }

        let vertex_bytes = self.vertices.len() * mem::size_of::<Vertex>();
        let index_bytes = self.indices.len() * mem::size_of::<u16>();

        // If vertex buffer too small -> recreate with new size (double strategy can help)
        if vertex_bytes > self.vbuf_size {
            // choose new size (double until big enough) to reduce realloc churn
            let mut new_vbuf_size = self.vbuf_size.max(1);
            while new_vbuf_size < vertex_bytes {
                new_vbuf_size *= 2;
            }
            // destroy old buffer and make a new one
            sg::destroy_buffer(self.bind.vertex_buffers[0]);
            let new_vbuf = sg::make_buffer(&sg::BufferDesc {
                size: new_vbuf_size,
                usage: sg::BufferUsage {
                    vertex_buffer: true,
                    stream_update: true,
                    ..Default::default()
                },
                ..Default::default()
            });
            self.bind.vertex_buffers[0] = new_vbuf;
            self.vbuf_size = new_vbuf_size;
        }

        if index_bytes > self.ibuf_size {
            let mut new_ibuf_size = self.ibuf_size.max(1);
            while new_ibuf_size < index_bytes {
                new_ibuf_size *= 2;
            }
            sg::destroy_buffer(self.bind.index_buffer);
            let new_ibuf = sg::make_buffer(&sg::BufferDesc {
                size: new_ibuf_size,
                usage: sg::BufferUsage {
                    index_buffer: true,
                    stream_update: true,
                    ..Default::default()
                },
                ..Default::default()
            });
            self.bind.index_buffer = new_ibuf;
            self.ibuf_size = new_ibuf_size;
        }

        // Update vertex buffer
        sg::update_buffer(
            self.bind.vertex_buffers[0],
            &sg::Range {
                ptr: self.vertices.as_ptr() as *const _,
                size: vertex_bytes,
            },
        );

        // Update index buffer
        sg::update_buffer(
            self.bind.index_buffer,
            &sg::Range {
                ptr: self.indices.as_ptr() as *const _,
                size: index_bytes,
            },
        );
        
        // Setup uniforms
        let view_proj = camera.get_view_projection_matrix();
        let uniforms = Uniforms {
            mvp: view_proj.to_cols_array_2d(),
        };

        // Draw all batches
        for batch in &self.batches {
            // Select pipeline based on whether we're using textures
            let uses_texture = batch.texture.id != self.texture_manager.get_white_texture().id;
            let pipeline = if uses_texture {
                self.textured_pipeline
            } else {
                self.colored_pipeline
            };

            // Bind texture and sampler
            let view = if let Some(&cached_view) = self.view_cache.get(&batch.texture.id) {
                cached_view
            } else {
                let new_view = sg::make_view(&sg::ViewDesc {
                    texture: sg::TextureViewDesc {
                        image: batch.texture,
                        ..Default::default()
                    },
                    ..Default::default()
                });
                self.view_cache.insert(batch.texture.id, new_view);
                new_view
            };
            
            self.bind.views[0] = view;

            self.bind.samplers[0] = self.sampler;

            // Apply pipeline and bindings
            sg::apply_pipeline(pipeline);
            sg::apply_bindings(&self.bind);
            sg::apply_uniforms(0, &sg::Range {
                ptr: &uniforms as *const _ as *const _,
                size: mem::size_of::<Uniforms>(),
            });

            // Draw this batch
            sg::draw(batch.start_index, batch.index_count, 1);
        }
    }

    fn add_batch(&mut self, texture: sg::Image, start_index: usize, index_count: usize) {
        // Check if we can merge with the last batch
        if let Some(last_batch) = self.batches.last_mut() {
            if last_batch.texture.id == texture.id {
                last_batch.index_count += index_count;
                return;
            }
        }

        // Create new batch
        self.batches.push(DrawBatch {
            texture,
            start_index,
            index_count,
        });
    }
}

/// Implementation for drawing to the screen used by the game
impl Renderer {
    pub fn draw_quad(&mut self, quad: &Quad) {
        let start_vertex = self.vertices.len() as u16;
        let start_index = self.indices.len();

        // Create 4 vertices for the quad
        let x1 = quad.position.x;
        let y1 = quad.position.y;
        let x2 = quad.position.x + quad.size.x;
        let y2 = quad.position.y + quad.size.y;
        
        let color = [quad.color.x, quad.color.y, quad.color.z, quad.color.w];

        // Add vertices (clockwise)
        self.vertices.push(Vertex { pos: [x1, y1], texcoord: [0.0, 0.0], color });
        self.vertices.push(Vertex { pos: [x2, y1], texcoord: [1.0, 0.0], color });
        self.vertices.push(Vertex { pos: [x2, y2], texcoord: [1.0, 1.0], color });
        self.vertices.push(Vertex { pos: [x1, y2], texcoord: [0.0, 1.0], color });
                
        // Add indices for two triangles
        let indices = [
            start_vertex, start_vertex + 1, start_vertex + 2,
            start_vertex, start_vertex + 2, start_vertex + 3,
        ];
        
        self.indices.extend_from_slice(&indices);

        self.add_batch(self.texture_manager.get_white_texture(), start_index, 6);
    }

    pub fn draw_circle(&mut self, circle: &Circle) {
        let center_vertex = self.vertices.len() as u16;
        let color = [circle.color.x, circle.color.y, circle.color.z, circle.color.w];
        
        // Add center vertex
        self.vertices.push(Vertex { 
            pos: [circle.center.x, circle.center.y], 
            texcoord: [0.5, 0.5],
            color 
        });
        
        // Add vertices around the circumference
        for i in 0..circle.segments {
            let angle = (i as f32 / circle.segments as f32) * 2.0 * std::f32::consts::PI;
            let x = circle.center.x + angle.cos() * circle.radius;
            let y = circle.center.y + angle.sin() * circle.radius;
            
            self.vertices.push( Vertex { 
                pos: [x, y],
                texcoord: [0.5, 0.5],
                    color 
            });
        }
        
        // Add triangles from center to each edge
        let triangle_count = circle.segments * 3;
        for i in 0..circle.segments {
            let next = (i + 1) % circle.segments;
            self.indices.extend_from_slice(&[
                center_vertex,                    // center
                center_vertex + 1 + i as u16,    // current point on circumference
                center_vertex + 1 + next as u16, // next point on circumference
            ]);
        }

        self.add_batch(self.texture_manager.get_white_texture(), center_vertex as usize, triangle_count as usize);
    }

    pub fn draw_sprite(&mut self, sprite: &Sprite) {
        let start_vertex = self.vertices.len() as u16;
        let start_index = self.indices.len();

        // Determine which texture to use
        // let texture = sprite.texture.unwrap_or(self.texture_manager.get_white_texture());
        let texture = self.get_texture(&sprite.texture_name).unwrap_or(self.texture_manager.get_white_texture());
        
        // Create 4 vertices for the sprite quad
        let half_size = sprite.size * 0.5;
        let cos_rot = sprite.rotation.cos();
        let sin_rot = sprite.rotation.sin();

        let local_positions = [
            Vec2::new(-half_size.x, -half_size.y), // Top-left
            Vec2::new(half_size.x, -half_size.y),  // Top-right
            Vec2::new(half_size.x, half_size.y),   // Bottom-right
            Vec2::new(-half_size.x, half_size.y),  // Bottom-left
        ];

        let mut uvs = [
            Vec2::new(sprite.uv.x, sprite.uv.y),                           // Top-left UV
            Vec2::new(sprite.uv.x + sprite.uv.z, sprite.uv.y),           // Top-right UV
            Vec2::new(sprite.uv.x + sprite.uv.z, sprite.uv.y + sprite.uv.w), // Bottom-right UV
            Vec2::new(sprite.uv.x, sprite.uv.y + sprite.uv.w),           // Bottom-left UV
        ];

        // Apply flipping by swapping UV coordinates
        if sprite.flip_x {
            uvs.swap(0, 1); // Swap top-left with top-right
            uvs.swap(2, 3); // Swap bottom-right with bottom-left
        }
        if sprite.flip_y {
            uvs.swap(0, 3); // Swap top-left with bottom-left
            uvs.swap(1, 2); // Swap top-right with bottom-right
        }

        let color = [sprite.color.x, sprite.color.y, sprite.color.z, sprite.color.w];

        // Add vertices with rotation applied
        for i in 0..4 {
            let local_pos = local_positions[i];
            
            // Apply rotation
            let rotated_pos = if sprite.rotation != 0.0 {
                Vec2::new(
                    local_pos.x * cos_rot - local_pos.y * sin_rot,
                    local_pos.x * sin_rot + local_pos.y * cos_rot,
                )
            } else {
                local_pos
            };

            // Apply world position
            let world_pos = sprite.position + rotated_pos;

            self.vertices.push(Vertex {
                pos: [world_pos.x, world_pos.y],
                texcoord: [uvs[i].x, uvs[i].y],
                color,
            });
        }

        // Add indices for two triangles
        let indices = [
            start_vertex, start_vertex + 1, start_vertex + 2,
            start_vertex, start_vertex + 2, start_vertex + 3,
        ];
        self.indices.extend_from_slice(&indices);
        self.add_batch(texture, start_index, 6);
    }

    // ADD texture loading method:
    pub fn load_texture(&mut self, name: &str, path: &str) -> Result<sg::Image, Box<dyn std::error::Error>> {
        self.texture_manager.load_texture(name, path)
    }

    pub fn get_texture(&self, name: &str) -> Option<sg::Image> {
        self.texture_manager.get_texture(name)
    }

    pub fn draw_particle(&mut self, particle: &Particle) {
        let size = 4.0; // Small particle size
        let alpha = particle.lifetime / particle.max_lifetime; // Fade out
        let color = Vec4::new(particle.color.x, particle.color.y, particle.color.z, alpha);
        
        let quad = Quad::new(
            particle.position.x - size * 0.5,
            particle.position.y - size * 0.5,
            size, size,
            color
        );
        self.draw_quad(&quad);
    }

}