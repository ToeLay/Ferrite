use std::sync::Arc;
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ObjectFit {
    #[default]
    Fill,
    Contain,
    Cover,
    ScaleDown,
}

#[derive(Clone, Debug)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub pixels: Arc<[u8]>, // RGBA8 (premultiplied for tiny-skia)
}

impl PartialEq for ImageData {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && Arc::ptr_eq(&self.pixels, &other.pixels)
    }
}

impl ImageData {
    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, image::ImageError> {
        let img = image::open(path)?;
        let rgba = img.into_rgba8();
        let (width, height) = rgba.dimensions();
        // We need to premultiply alpha for tiny-skia.
        let mut pixels = rgba.into_raw();
        for chunk in pixels.chunks_exact_mut(4) {
            let a = chunk[3] as u32;
            if a < 255 {
                chunk[0] = ((chunk[0] as u32 * a) / 255) as u8;
                chunk[1] = ((chunk[1] as u32 * a) / 255) as u8;
                chunk[2] = ((chunk[2] as u32 * a) / 255) as u8;
            }
        }
        Ok(Self {
            width,
            height,
            pixels: pixels.into(),
        })
    }
}
