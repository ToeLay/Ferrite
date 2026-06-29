use ferrite::prelude::*;

fn main() {
    // Generate a simple test image programmatically.
    let img_path = "test_image.png";
    if !std::path::Path::new(img_path).exists() {
        let mut img = ::image::ImageBuffer::new(200, 200);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let r = (x as f32 / 200.0 * 255.0) as u8;
            let g = (y as f32 / 200.0 * 255.0) as u8;
            let b = 128;
            *pixel = ::image::Rgba([r, g, b, 255]);
        }
        img.save(img_path).unwrap();
    }

    let img_data = ImageData::load_from_path(img_path).expect("Failed to load image");

    ferrite::run("Image Example", (500, 600), col([
        text("Image Widget Example").size(24.0).color(Color::rgb(1.0, 1.0, 1.0)),
        
        text("ObjectFit::Fill (Stretched)").size(16.0).color(Color::rgb(0.8, 0.8, 0.8)),
        image(img_data.clone())
            .object_fit(ObjectFit::Fill)
            .corner_radius(12.0)
            .width(300.0)
            .height(100.0),
            
        text("ObjectFit::Cover (Cropped)").size(16.0).color(Color::rgb(0.8, 0.8, 0.8)),
        image(img_data.clone())
            .object_fit(ObjectFit::Cover)
            .corner_radius(12.0)
            .width(300.0)
            .height(100.0),
            
        text("ObjectFit::Contain (Scaled)").size(16.0).color(Color::rgb(0.8, 0.8, 0.8)),
        col([
            image(img_data)
                .object_fit(ObjectFit::Contain)
                .corner_radius(12.0)
                .fill()
        ])
        .width(300.0)
        .height(100.0)
        .background(Color::rgb(0.2, 0.2, 0.2))
        .corner_radius(12.0),
    ])
    .gap(15.0)
    .padding(20.0)
    .align(AlignItems::Center)
    .background(Color::rgb(0.12, 0.12, 0.12)));
}
