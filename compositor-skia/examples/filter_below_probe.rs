use std::sync::Arc;

use compositor::{Filter, FilterBelowLayer, Geometry, Layer, PictureLayer, Point, Radius, Rectangle, RoundedRectangle};
use compositor_skia::{Cache, SkiaCompositor, SkiaPicture};
use skia_safe::{canvas::SaveLayerRec, image_filters, surfaces, ClipOp, Color, IPoint, ISize, ImageInfo, Paint, PictureRecorder, RRect, Rect};

const WIDTH: i32 = 520;
const HEIGHT: i32 = 340;
const PANEL_X: f32 = 110.0;
const PANEL_Y: f32 = 85.0;
const PANEL_W: f32 = 300.0;
const PANEL_H: f32 = 170.0;
const CORNER: f32 = 20.0;
const STRIPE_W: f32 = 16.0;

fn draw_vertical_stripes(canvas: &skia_safe::Canvas) {
    let mut paint = Paint::default();
    let mut x = 0.0;
    let mut index = 0;

    while x < WIDTH as f32 {
        paint.set_color(if index % 2 == 0 { Color::RED } else { Color::BLUE });
        canvas.draw_rect(Rect::from_xywh(x, 0.0, STRIPE_W, HEIGHT as f32), &paint);
        x += STRIPE_W;
        index += 1;
    }
}

fn vertical_stripes_picture() -> skia_safe::Picture {
    let mut recorder = PictureRecorder::new();
    let canvas = recorder.begin_recording(Rect::from_xywh(0.0, 0.0, WIDTH as f32, HEIGHT as f32), None);
    draw_vertical_stripes(canvas);
    recorder.finish_recording_as_picture(None).expect("picture")
}

fn draw_panel_fill(canvas: &skia_safe::Canvas, rect: Rect) {
    let mut paint = Paint::default();
    paint.set_color(Color::from_argb(70, 0, 0, 0));
    paint.set_anti_alias(true);
    canvas.draw_rrect(RRect::new_rect_xy(rect, CORNER, CORNER), &paint);
}

fn draw_direct_backdrop(canvas: &skia_safe::Canvas, rect: Rect, sigma: f32, draw_fill: bool) {
    let blur = image_filters::blur((sigma, sigma), None, None, None)
        .expect("Failed to create blur filter");

    canvas.save();
    canvas.clip_rrect(RRect::new_rect_xy(rect, CORNER, CORNER), ClipOp::Intersect, true);
    let save_layer_rec = SaveLayerRec::default().backdrop(&blur);
    canvas.save_layer(&save_layer_rec);
    canvas.restore();
    if draw_fill {
        draw_panel_fill(canvas, rect);
    }
    canvas.restore();
}

fn filter_below_layer(rect: Rect, sigma: f32) -> FilterBelowLayer {
    let geometry = Geometry::RoundedRectangle(RoundedRectangle::new(
        Rectangle::extent(rect.width(), rect.height()),
        Radius::new(CORNER, CORNER),
        Radius::new(CORNER, CORNER),
        Radius::new(CORNER, CORNER),
        Radius::new(CORNER, CORNER),
    ));

    FilterBelowLayer::new(
        Filter::blur(Radius::new(sigma, sigma)),
        geometry,
        Point::new_f32(rect.left, rect.top),
    )
}

fn compose_filter_below(canvas: &skia_safe::Canvas, rect: Rect, sigma: f32, draw_fill: bool) {
    let mut cache = Cache::new();
    let mut compositor = SkiaCompositor::new(None, canvas, &mut cache);
    Arc::new(filter_below_layer(rect, sigma)).compose(&mut compositor);
    if draw_fill {
        draw_panel_fill(canvas, rect);
    }
}

fn compose_picture_then_filter_below(canvas: &skia_safe::Canvas, rect: Rect, sigma: f32, draw_fill: bool, needs_cache: bool) {
    let mut cache = Cache::new();
    let mut compositor = SkiaCompositor::new(None, canvas, &mut cache);

    let picture = Arc::new(SkiaPicture::new(vertical_stripes_picture())) as Arc<dyn compositor::Picture>;
    Arc::new(PictureLayer::new(picture, needs_cache)).compose(&mut compositor);
    Arc::new(filter_below_layer(rect, sigma)).compose(&mut compositor);

    if draw_fill {
        draw_panel_fill(canvas, rect);
    }
}

fn surface_pixels<F>(draw_base: bool, draw: F) -> Vec<u8>
where
    F: FnOnce(&skia_safe::Canvas),
{
    let mut surface = surfaces::raster_n32_premul(ISize::new(WIDTH, HEIGHT)).unwrap();
    let canvas = surface.canvas();
    canvas.clear(Color::WHITE);
    if draw_base {
        draw_vertical_stripes(canvas);
    }
    draw(canvas);

    let info = ImageInfo::new_n32_premul(ISize::new(WIDTH, HEIGHT), None);
    let mut pixels = vec![0; (WIDTH * HEIGHT * 4) as usize];
    surface.read_pixels(&info, pixels.as_mut_slice(), (WIDTH * 4) as usize, IPoint::new(0, 0));
    pixels
}

fn summarize_line(label: &str, pixels: &[u8], points: impl Iterator<Item = (i32, i32)>, axis: &str) {
    let mut min = [255u8; 4];
    let mut max = [0u8; 4];
    let mut transitions = 0u32;
    let mut previous: Option<[u8; 4]> = None;
    let mut samples = Vec::new();

    for (x, y) in points {
        let index = ((y * WIDTH + x) * 4) as usize;
        let pixel = [pixels[index], pixels[index + 1], pixels[index + 2], pixels[index + 3]];

        if samples.last() != Some(&pixel) {
            samples.push(pixel);
        }

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

    let preview: Vec<[u8; 4]> = samples.iter().copied().take(8).collect();
    println!(
        "{label}: {axis}, min={min:?}, max={max:?}, strong_transitions={transitions}, first_runs={preview:?}"
    );
}

fn summarize(label: &str, pixels: &[u8]) {
    let y = (PANEL_Y + PANEL_H / 2.0) as i32;
    let x0 = PANEL_X as i32 + 10;
    let x1 = (PANEL_X + PANEL_W) as i32 - 10;
    summarize_line(
        label,
        pixels,
        (x0..x1).map(|x| (x, y)),
        &format!("horizontal row across vertical stripes y={y}, x={x0}..{x1}"),
    );
}

fn main() {
    let rect = Rect::from_xywh(PANEL_X, PANEL_Y, PANEL_W, PANEL_H);

    for sigma in [0.0, 2.0, 8.0, 18.0] {
        let direct = surface_pixels(true, |canvas| draw_direct_backdrop(canvas, rect, sigma, false));
        summarize(&format!("direct backdrop no-fill sigma={sigma}"), &direct);

        let compositor_over_direct_background = surface_pixels(true, |canvas| compose_filter_below(canvas, rect, sigma, false));
        summarize(&format!("filter-below over directly drawn background no-fill sigma={sigma}"), &compositor_over_direct_background);

        let picture_uncached = surface_pixels(false, |canvas| compose_picture_then_filter_below(canvas, rect, sigma, false, false));
        summarize(&format!("picture layer uncached then filter-below no-fill sigma={sigma}"), &picture_uncached);

        let picture_cached = surface_pixels(false, |canvas| compose_picture_then_filter_below(canvas, rect, sigma, false, true));
        summarize(&format!("picture layer cached then filter-below no-fill sigma={sigma}"), &picture_cached);
    }
}
