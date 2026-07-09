use std::sync::Arc;

use compositor::{Filter, FilterBelowLayer, Geometry, Layer, Point, Radius, Rectangle, RoundedRectangle};
use compositor_skia::{Cache, SkiaCompositor};
use skia_safe::{canvas::SaveLayerRec, image_filters, surfaces, ClipOp, Color, Color4f, IPoint, ISize, ImageInfo, Paint, RRect, Rect};

const WIDTH: i32 = 420;
const HEIGHT: i32 = 260;
const PANEL_X: f32 = 90.0;
const PANEL_Y: f32 = 70.0;
const PANEL_W: f32 = 240.0;
const PANEL_H: f32 = 130.0;
const CORNER: f32 = 20.0;

fn draw_stripes(canvas: &skia_safe::Canvas) {
    let mut paint = Paint::default();
    let stripe_height = 20.0;
    let mut y = 0.0;
    let mut index = 0;

    while y < HEIGHT as f32 {
        paint.set_color(if index % 2 == 0 { Color::RED } else { Color::BLUE });
        canvas.draw_rect(Rect::from_xywh(0.0, y, WIDTH as f32, stripe_height), &paint);
        y += stripe_height;
        index += 1;
    }
}

fn draw_panel_fill(canvas: &skia_safe::Canvas, rect: Rect) {
    let mut paint = Paint::default();
    paint.set_color(Color::from_argb(70, 0, 0, 0));
    paint.set_anti_alias(true);
    canvas.draw_rrect(RRect::new_rect_xy(rect, CORNER, CORNER), &paint);
}

fn draw_direct_backdrop(canvas: &skia_safe::Canvas, rect: Rect, sigma: f32) {
    let blur = image_filters::blur((sigma, sigma), None, None, None)
        .expect("Failed to create blur filter");

    canvas.save();
    canvas.clip_rrect(RRect::new_rect_xy(rect, CORNER, CORNER), ClipOp::Intersect, true);
    let save_layer_rec = SaveLayerRec::default().backdrop(&blur);
    canvas.save_layer(&save_layer_rec);
    canvas.restore();
    draw_panel_fill(canvas, rect);
    canvas.restore();
}

fn draw_compositor_backdrop(canvas: &skia_safe::Canvas, rect: Rect, sigma: f32) {
    let geometry = Geometry::RoundedRectangle(RoundedRectangle::new(
        Rectangle::extent(rect.width(), rect.height()),
        Radius::new(CORNER, CORNER),
        Radius::new(CORNER, CORNER),
        Radius::new(CORNER, CORNER),
        Radius::new(CORNER, CORNER),
    ));

    let layer = FilterBelowLayer::new(
        Filter::blur(Radius::new(sigma, sigma)),
        geometry,
        Point::new_f32(rect.left, rect.top),
    );

    let mut cache = Cache::new();
    let mut compositor = SkiaCompositor::new(None, canvas, &mut cache);
    Arc::new(layer).compose(&mut compositor);
    draw_panel_fill(canvas, rect);
}

fn surface_pixels<F>(draw: F) -> Vec<u8>
where
    F: FnOnce(&skia_safe::Canvas),
{
    let mut surface = surfaces::raster_n32_premul(ISize::new(WIDTH, HEIGHT)).unwrap();
    let canvas = surface.canvas();
    canvas.clear(Color::WHITE);
    draw_stripes(canvas);
    draw(canvas);

    let info = ImageInfo::new_n32_premul(ISize::new(WIDTH, HEIGHT), None);
    let mut pixels = vec![0; (WIDTH * HEIGHT * 4) as usize];
    surface.read_pixels(&info, pixels.as_mut_slice(), (WIDTH * 4) as usize, IPoint::new(0, 0));
    pixels
}

fn summarize(label: &str, pixels: &[u8]) {
    let x0 = PANEL_X as i32 + 20;
    let x1 = (PANEL_X + PANEL_W) as i32 - 20;
    let y = (PANEL_Y + PANEL_H / 2.0) as i32;

    let mut min = [255u8; 4];
    let mut max = [0u8; 4];
    let mut transitions = 0u32;
    let mut previous: Option<[u8; 4]> = None;

    for x in x0..x1 {
        let index = ((y * WIDTH + x) * 4) as usize;
        let pixel = [pixels[index], pixels[index + 1], pixels[index + 2], pixels[index + 3]];

        for channel in 0..4 {
            min[channel] = min[channel].min(pixel[channel]);
            max[channel] = max[channel].max(pixel[channel]);
        }

        if let Some(previous) = previous {
            let delta: u32 = (0..4)
                .map(|channel| previous[channel].abs_diff(pixel[channel]) as u32)
                .sum();
            if delta > 20 {
                transitions += 1;
            }
        }

        previous = Some(pixel);
    }

    println!(
        "{label}: row y={y}, x={x0}..{x1}, min={min:?}, max={max:?}, strong_transitions={transitions}"
    );
}

fn main() {
    let rect = Rect::from_xywh(PANEL_X, PANEL_Y, PANEL_W, PANEL_H);

    let control = surface_pixels(|canvas| draw_panel_fill(canvas, rect));
    summarize("control fill only", &control);

    for sigma in [0.0, 0.5, 2.0, 8.0] {
        let direct = surface_pixels(|canvas| draw_direct_backdrop(canvas, rect, sigma));
        summarize(&format!("direct backdrop sigma={sigma}"), &direct);
    }

    for sigma in [0.0, 0.5, 2.0, 8.0] {
        let compositor = surface_pixels(|canvas| draw_compositor_backdrop(canvas, rect, sigma));
        summarize(&format!("compositor filter-below sigma={sigma}"), &compositor);
    }
}
