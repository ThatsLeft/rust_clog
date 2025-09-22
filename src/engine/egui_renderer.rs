// src/engine/egui_renderer.rs

use crate::engine::graphics::Vertex;
use glam::Vec2;
use sokol::gfx as sg;
use std::collections::HashMap;

pub struct EguiRenderer {
    pipeline: sg::Pipeline,
    bind: sg::Bindings,
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    textures: HashMap<egui::TextureId, sg::Image>,
    font_texture: Option<sg::Image>,
    sampler: sg::Sampler,
    font_view: Option<sg::View>,
}

impl EguiRenderer {
    pub fn new() -> Self {
        // Create vertex buffer (will be updated each frame)
        let vbuf = sg::make_buffer(&sg::BufferDesc {
            usage: sg::BufferUsage {
                vertex_buffer: true,
                stream_update: true, // This buffer will be updated every frame
                ..Default::default()
            },
            size: 65536 * std::mem::size_of::<Vertex>(), // Max vertices
            ..Default::default()
        });

        // Create index buffer (will be updated each frame)
        let ibuf = sg::make_buffer(&sg::BufferDesc {
            usage: sg::BufferUsage {
                index_buffer: true,
                stream_update: true, // This buffer will be updated every frame
                ..Default::default()
            },
            size: 65536 * std::mem::size_of::<u16>(), // Max indices
            ..Default::default()
        });

        // Create sampler for UI textures
        let sampler = sg::make_sampler(&sg::SamplerDesc {
            min_filter: sg::Filter::Linear,
            mag_filter: sg::Filter::Linear,
            wrap_u: sg::Wrap::ClampToEdge,
            wrap_v: sg::Wrap::ClampToEdge,
            ..Default::default()
        });

        let mut bind = sg::Bindings::new();
        bind.vertex_buffers[0] = vbuf;
        bind.index_buffer = ibuf;
        bind.samplers[0] = sampler;

        // Create pipeline for UI rendering
        let pipeline = Self::create_ui_pipeline();

        Self {
            pipeline,
            bind,
            vertices: Vec::new(),
            indices: Vec::new(),
            textures: HashMap::new(),
            font_texture: None,
            sampler,
            font_view: None,
        }
    }

    fn create_ui_pipeline() -> sg::Pipeline {
        // Shader source for egui rendering
        let vs_source = "
cbuffer uniforms : register(b0) {
    float2 u_screen_size;
};

struct vs_in {
    float2 position : POSITION;
    float2 texcoord : TEXCOORD;
    float4 color : COLOR;
};

struct vs_out {
    float4 position : SV_Position;
    float2 texcoord : TEXCOORD;
    float4 color : COLOR;
};

vs_out main(vs_in inp) {
    vs_out outp;
    float2 ndc = 2.0 * inp.position / u_screen_size - 1.0;
    ndc.y = -ndc.y;
    outp.position = float4(ndc, 0.0, 1.0);
    outp.texcoord = inp.texcoord;
    outp.color = inp.color;
    return outp;
}
\0";

        let fs_source = "
Texture2D u_texture : register(t0);
SamplerState u_sampler : register(s0);

struct ps_in {
    float4 position : SV_Position;
    float2 texcoord : TEXCOORD;
    float4 color : COLOR;
};

float4 main(ps_in inp) : SV_Target0 {
    return inp.color * u_texture.Sample(u_sampler, inp.texcoord);
}
\0";

        let shader_desc = sg::ShaderDesc {
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
                    size: 8,                  // 2 floats for screen_size (Vec2 = 8 bytes)
                    hlsl_register_b_n: 0,     // matches "register(b0)" in shader
                    msl_buffer_n: 0,          // Metal buffer slot
                    wgsl_group0_binding_n: 0, // WebGPU binding
                    layout: sg::UniformLayout::Std140,
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
            views: [
                sg::ShaderView {
                    texture: sg::ShaderTextureView {
                        stage: sg::ShaderStage::Fragment,
                        image_type: sg::ImageType::Dim2, // 2D texture
                        sample_type: sg::ImageSampleType::Float,
                        multisampled: false,
                        hlsl_register_t_n: 0, // Maps to register(t0) in HLSL
                        msl_texture_n: 0,
                        wgsl_group1_binding_n: 0,
                        ..Default::default()
                    },
                    storage_buffer: sg::ShaderStorageBufferView::default(),
                    storage_image: sg::ShaderStorageImageView::default(),
                },
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
                Default::default(),
            ],
            texture_sampler_pairs: [
                sg::ShaderTextureSamplerPair {
                    stage: sg::ShaderStage::Fragment,
                    view_slot: 0,
                    sampler_slot: 0,
                    glsl_name: std::ptr::null(),
                },
                sg::ShaderTextureSamplerPair::new(),
                sg::ShaderTextureSamplerPair::new(),
                sg::ShaderTextureSamplerPair::new(),
                sg::ShaderTextureSamplerPair::new(),
                sg::ShaderTextureSamplerPair::new(),
                sg::ShaderTextureSamplerPair::new(),
                sg::ShaderTextureSamplerPair::new(),
                sg::ShaderTextureSamplerPair::new(),
                sg::ShaderTextureSamplerPair::new(),
                sg::ShaderTextureSamplerPair::new(),
                sg::ShaderTextureSamplerPair::new(),
                sg::ShaderTextureSamplerPair::new(),
                sg::ShaderTextureSamplerPair::new(),
                sg::ShaderTextureSamplerPair::new(),
                sg::ShaderTextureSamplerPair::new(),
            ],
            samplers: [
                sg::ShaderSampler {
                    stage: sg::ShaderStage::Fragment,
                    sampler_type: sg::SamplerType::Filtering,
                    ..Default::default()
                },
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
            ..Default::default()
        };

        let shader = sg::make_shader(&shader_desc);

        // Pipeline layout description matching your Vertex struct
        let mut pipeline_desc = sg::PipelineDesc {
            shader,
            index_type: sg::IndexType::Uint16,
            primitive_type: sg::PrimitiveType::Triangles,
            ..Default::default()
        };

        // Set up vertex layout to match your Vertex struct
        pipeline_desc.layout.attrs[0] = sg::VertexAttrState {
            format: sg::VertexFormat::Float2, // position
            buffer_index: 0,
            offset: 0,
        };
        pipeline_desc.layout.attrs[1] = sg::VertexAttrState {
            format: sg::VertexFormat::Float2, // tex_coord
            buffer_index: 0,
            offset: 8, // 2 floats = 8 bytes
        };
        pipeline_desc.layout.attrs[2] = sg::VertexAttrState {
            format: sg::VertexFormat::Float4, // color
            buffer_index: 0,
            offset: 16, // 2+2 floats = 16 bytes
        };

        // Set buffer stride
        pipeline_desc.layout.buffers[0] = sg::VertexBufferLayoutState {
            stride: std::mem::size_of::<Vertex>() as i32, // 32 bytes total
            step_func: sg::VertexStep::PerVertex,
            step_rate: 1,
        };

        // Enable alpha blending for UI
        pipeline_desc.colors[0] = sg::ColorTargetState {
            blend: sg::BlendState {
                enabled: true,
                src_factor_rgb: sg::BlendFactor::SrcAlpha,
                dst_factor_rgb: sg::BlendFactor::OneMinusSrcAlpha,
                op_rgb: sg::BlendOp::Add,
                src_factor_alpha: sg::BlendFactor::One,
                dst_factor_alpha: sg::BlendFactor::OneMinusSrcAlpha,
                op_alpha: sg::BlendOp::Add,
            },
            ..Default::default()
        };

        sg::make_pipeline(&pipeline_desc)
    }

    pub fn update_textures(&mut self, textures_delta: &egui::TexturesDelta) {
        // Update all textures including font
        for (id, image_delta) in &textures_delta.set {
            self.update_texture(*id, image_delta);
        }

        // Remove old textures
        for id in &textures_delta.free {
            if let Some(image) = self.textures.remove(id) {
                sg::destroy_image(image);
            }
        }
    }

    fn update_texture(&mut self, id: egui::TextureId, image_delta: &egui::epaint::ImageDelta) {
        let image = match &image_delta.image {
            egui::ImageData::Color(color_image) => {
                if id == egui::TextureId::default() {
                    // Font texture - convert RGBA to alpha-only
                    let alpha_pixels: Vec<f32> = color_image
                        .pixels
                        .iter()
                        .map(|color| color.a() as f32 / 255.0)
                        .collect();
                    self.create_alpha_texture(
                        color_image.width(),
                        color_image.height(),
                        &alpha_pixels,
                    )
                } else {
                    // Regular RGBA texture
                    self.create_rgba_texture(
                        color_image.width(),
                        color_image.height(),
                        &color_image.pixels,
                    )
                }
            }
        };

        if id == egui::TextureId::default() {
            self.font_texture = Some(image);
            self.font_view = Some(sg::make_view(&sg::ViewDesc {
                texture: sg::TextureViewDesc {
                    image,
                    ..Default::default()
                },
                ..Default::default()
            }));
        }
        self.textures.insert(id, image);
    }

    fn create_rgba_texture(
        &self,
        width: usize,
        height: usize,
        pixels: &[egui::Color32],
    ) -> sg::Image {
        let rgba_pixels: Vec<u8> = pixels
            .iter()
            .flat_map(|color| [color.r(), color.g(), color.b(), color.a()])
            .collect();

        let mut desc = sg::ImageDesc::new();
        desc.width = width as i32;
        desc.height = height as i32;
        desc.pixel_format = sg::PixelFormat::Rgba8;
        desc.data.subimage[0][0] = sg::Range {
            ptr: rgba_pixels.as_ptr() as *const _,
            size: rgba_pixels.len(),
        };

        sg::make_image(&desc)
    }

    fn create_alpha_texture(&self, width: usize, height: usize, pixels: &[f32]) -> sg::Image {
        let alpha_pixels: Vec<u8> = pixels.iter().map(|alpha| (*alpha * 255.0) as u8).collect();

        let mut desc = sg::ImageDesc::new();
        desc.width = width as i32;
        desc.height = height as i32;
        desc.pixel_format = sg::PixelFormat::R8;
        desc.data.subimage[0][0] = sg::Range {
            ptr: alpha_pixels.as_ptr() as *const _,
            size: alpha_pixels.len(),
        };

        sg::make_image(&desc)
    }

    pub fn render(&mut self, shapes: &[egui::epaint::ClippedPrimitive], screen_size: Vec2) {
        if shapes.is_empty() {
            return;
        }

        self.vertices.clear();
        self.indices.clear();

        // Convert egui primitives to our vertex format
        for clipped_primitive in shapes {
            let clip_rect = clipped_primitive.clip_rect;

            // Skip primitives that are completely outside the screen
            if clip_rect.max.x < 0.0
                || clip_rect.max.y < 0.0
                || clip_rect.min.x > screen_size.x
                || clip_rect.min.y > screen_size.y
            {
                continue;
            }

            if let egui::epaint::Primitive::Mesh(mesh) = &clipped_primitive.primitive {
                let vertex_start = self.vertices.len() as u16;

                // Convert vertices
                for vertex in &mesh.vertices {
                    self.vertices.push(Vertex {
                        pos: [vertex.pos.x, vertex.pos.y],
                        texcoord: [vertex.uv.x, vertex.uv.y],
                        color: [
                            vertex.color.r() as f32 / 255.0,
                            vertex.color.g() as f32 / 255.0,
                            vertex.color.b() as f32 / 255.0,
                            vertex.color.a() as f32 / 255.0,
                        ],
                    });
                }

                // Convert indices
                for &idx in &mesh.indices {
                    self.indices.push(vertex_start + idx as u16);
                }
            }
        }

        if self.vertices.is_empty() {
            return;
        }

        // Update vertex buffer
        sg::update_buffer(
            self.bind.vertex_buffers[0],
            &sg::Range {
                ptr: self.vertices.as_ptr() as *const _,
                size: self.vertices.len() * std::mem::size_of::<Vertex>(),
            },
        );

        // Update index buffer
        sg::update_buffer(
            self.bind.index_buffer,
            &sg::Range {
                ptr: self.indices.as_ptr() as *const _,
                size: self.indices.len() * std::mem::size_of::<u16>(),
            },
        );

        // Apply pipeline
        sg::apply_pipeline(self.pipeline);

        // Apply bindings with font texture
        if let Some(font_view) = self.font_view {
            self.bind.views[0] = font_view;
        }

        sg::apply_bindings(&self.bind);

        // Set screen size uniform
        let uniforms = [screen_size.x, screen_size.y];
        sg::apply_uniforms(
            0,
            &sg::Range {
                ptr: uniforms.as_ptr() as *const _,
                size: std::mem::size_of_val(&uniforms),
            },
        );

        // Draw
        sg::draw(0, self.indices.len(), 1);
    }
}
