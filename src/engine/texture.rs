use sokol::gfx as sg;
use std::collections::HashMap;

pub struct TextureManager {
    textures: HashMap<String, sg::Image>,
    white_texture: sg::Image,
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            white_texture: sg::Image::default(),
        }
    }

    pub fn init(&mut self) {
        let white_pixels = [255u8, 255, 255, 255];
        self.white_texture = sg::make_image(&sg::ImageDesc {
            width: 1,
            height: 1,
            data: sg::ImageData {
                subimage: [[sg::Range {
                    ptr: white_pixels.as_ref().as_ptr() as *const _,
                    size: white_pixels.as_ref().len(),
                }; 16]; 6],
            },
            ..Default::default()
        });
    }

    pub fn load_texture(&mut self, name: &str, path: &str) -> Result<sg::Image, Box<dyn std::error::Error>> {
        // Check if already loaded
        if let Some(&texture) = self.textures.get(name) {
            return Ok(texture);
        }

        // Load image file
        let img = image::open(path)?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        // Create sokol texture
        let sg_texture = sg::make_image(&sg::ImageDesc {
            width: width as i32,
            height: height as i32,
            pixel_format: sg::PixelFormat::Rgba8,
            data: sg::ImageData {
                subimage: [[sg::Range {
                    ptr: rgba.as_raw().as_ptr() as *const _,
                    size: rgba.as_raw().len(),
                }; 16]; 6],
            },
            ..Default::default()
        });

        // Store in cache
        self.textures.insert(name.to_string(), sg_texture);
        Ok(sg_texture)
    }

    pub fn get_texture(&self, name: &str) -> Option<sg::Image> {
        self.textures.get(name).copied()
    }

    pub fn get_white_texture(&self) -> sg::Image {
        self.white_texture
    }
}