mod driver;

use std::sync::Arc;

use compositor::{Filter, FilterBelowLayer, Geometry, Layer, Point, Radius, Rectangle, RoundedRectangle};
use compositor_skia::{Cache, SkiaCompositor};
use skia_safe::{canvas::SaveLayerRec, image_filters, ClipOp, Color, Paint, RRect, Rect};

fn draw_stripes(canvas: &skia_safe::Canvas, bounds: Rect) {
    let mut paint = Paint::default();
    let stripe_height = 20.0;
    let mut y = bounds.top;
    let mut index = 0;

    while y < bounds.bottom {
        paint.set_color(if index % 2 == 0 { Color::RED } else { Color::BLUE });
        canvas.draw_rect(
            Rect::from_xywh(bounds.left, y, bounds.width(), stripe_height),
            &paint,
        );
        y += stripe_height;
        index += 1;
    }
}

fn draw_panel_fill(canvas: &skia_safe::Canvas, rect: Rect) {
    let mut paint = Paint::default();
    paint.set_color(Color::from_argb(70, 0, 0, 0));
    paint.set_anti_alias(true);
    canvas.draw_rrect(RRect::new_rect_xy(rect, 20.0, 20.0), &paint);
}

fn draw_direct_save_layer_backdrop(canvas: &skia_safe::Canvas, rect: Rect, sigma: f32) {
    let blur = image_filters::blur((sigma, sigma), None, None, None)
        .expect("Failed to create blur filter");

    canvas.save();
    canvas.clip_rrect(RRect::new_rect_xy(rect, 20.0, 20.0), ClipOp::Intersect, true);
    let save_layer_rec = SaveLayerRec::default().backdrop(&blur);
    canvas.save_layer(&save_layer_rec);
    canvas.restore();
    draw_panel_fill(canvas, rect);
    canvas.restore();
}

fn draw_compositor_filter_below(canvas: &skia_safe::Canvas, rect: Rect, sigma: f32) {
    let geometry = Geometry::RoundedRectangle(RoundedRectangle::new(
        Rectangle::extent(rect.width(), rect.height()),
        Radius::new(20.0, 20.0),
        Radius::new(20.0, 20.0),
        Radius::new(20.0, 20.0),
        Radius::new(20.0, 20.0),
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

fn main() {
    env_logger::init();

    driver::run(move |canvas| {
        canvas.clear(Color::WHITE);
        draw_stripes(canvas, Rect::from_xywh(0.0, 0.0, 800.0, 600.0));

        // Left: direct Skia SaveLayerRec::backdrop usage.
        draw_direct_save_layer_backdrop(canvas, Rect::from_xywh(70.0, 170.0, 260.0, 160.0), 2.0);

        // Right: compositor FilterBelowLayer using the same Skia primitive internally.
        draw_compositor_filter_below(canvas, Rect::from_xywh(470.0, 170.0, 260.0, 160.0), 2.0);
    });
}
