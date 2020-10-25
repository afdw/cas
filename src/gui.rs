use crate::{data::*, Value};
use cairo::{Content, Context, Format, ImageSurface, Pattern, RecordingSurface, SurfacePattern};
use gio::prelude::*;
use gtk::{prelude::*, Application, ApplicationWindow, DrawingArea};
use itertools::Itertools;

#[derive(Clone)]
struct RenderResult {
    pattern: Pattern,
    width: f64,
    height: f64,
}

fn render_empty(width: f64, height: f64) -> RenderResult {
    let cr = Context::new(&*RecordingSurface::create(Content::ColorAlpha, None).unwrap());
    RenderResult {
        pattern: (&*SurfacePattern::create(&cr.get_target())).clone(),
        width,
        height,
    }
}

fn render_text(text: &str) -> RenderResult {
    let cr = Context::new(&*RecordingSurface::create(Content::ColorAlpha, None).unwrap());
    let text_extents = cr.text_extents(text);
    cr.save();
    cr.move_to(-text_extents.x_bearing, -text_extents.y_bearing);
    cr.show_text(text);
    cr.restore();
    RenderResult {
        pattern: (&*SurfacePattern::create(&cr.get_target())).clone(),
        width: text_extents.width,
        height: text_extents.height,
    }
}

fn render_underline(width: f64, red: f64, green: f64, blue: f64) -> RenderResult {
    let cr = Context::new(&*RecordingSurface::create(Content::ColorAlpha, None).unwrap());
    cr.set_line_width(1.0);
    cr.set_source_rgb(red, green, blue);
    cr.move_to(0.0, 1.5);
    cr.line_to(width, 1.0);
    cr.stroke();
    RenderResult {
        pattern: (&*SurfacePattern::create(&cr.get_target())).clone(),
        width,
        height: 2.0,
    }
}

fn render_scaled(render_result: RenderResult, sx: f64, sy: f64) -> RenderResult {
    let cr = Context::new(&*RecordingSurface::create(Content::ColorAlpha, None).unwrap());
    cr.scale(sx, sy);
    cr.set_source(&render_result.pattern);
    cr.paint();
    RenderResult {
        pattern: (&*SurfacePattern::create(&cr.get_target())).clone(),
        width: render_result.width * sx,
        height: render_result.height * sy,
    }
}

fn render_rasterized(render_result: RenderResult) -> RenderResult {
    let cr = Context::new(&*ImageSurface::create(Format::ARgb32, render_result.width.ceil() as i32, render_result.height.ceil() as i32).unwrap());
    cr.set_source(&render_result.pattern);
    cr.paint();
    RenderResult {
        pattern: (&*SurfacePattern::create(&cr.get_target())).clone(),
        width: render_result.width,
        height: render_result.height,
    }
}

#[allow(dead_code)]
enum ComponentsLayout {
    Top,
    Middle,
    Bottom,
    Left,
    Center,
    Right,
}

fn render_components(layout: ComponentsLayout, components: &[RenderResult]) -> RenderResult {
    let cr = Context::new(&*RecordingSurface::create(Content::ColorAlpha, None).unwrap());
    cr.save();
    let max_height = components.iter().map(|render_result| render_result.height).fold_first(f64::max).unwrap_or(0.0);
    let max_width = components.iter().map(|render_result| render_result.width).fold_first(f64::max).unwrap_or(0.0);
    match layout {
        ComponentsLayout::Top => cr.translate(0.0, 0.0),
        ComponentsLayout::Middle => cr.translate(0.0, max_height / 2.0),
        ComponentsLayout::Bottom => cr.translate(0.0, max_height),
        ComponentsLayout::Left => cr.translate(0.0, 0.0),
        ComponentsLayout::Center => cr.translate(max_width / 2.0, 0.0),
        ComponentsLayout::Right => cr.translate(max_width, 0.0),
    }
    for render_result in components {
        cr.save();
        match layout {
            ComponentsLayout::Top => cr.translate(0.0, 0.0),
            ComponentsLayout::Middle => cr.translate(0.0, -render_result.height / 2.0),
            ComponentsLayout::Bottom => cr.translate(0.0, -render_result.height),
            ComponentsLayout::Left => cr.translate(0.0, 0.0),
            ComponentsLayout::Center => cr.translate(-render_result.width / 2.0, 0.0),
            ComponentsLayout::Right => cr.translate(-render_result.width, 0.0),
        }
        cr.set_source(&render_result.pattern);
        cr.paint();
        cr.restore();
        match layout {
            ComponentsLayout::Top | ComponentsLayout::Middle | ComponentsLayout::Bottom => cr.translate(render_result.width, 0.0),
            ComponentsLayout::Left | ComponentsLayout::Center | ComponentsLayout::Right => cr.translate(0.0, render_result.height),
        }
    }
    cr.restore();
    match layout {
        ComponentsLayout::Top | ComponentsLayout::Middle | ComponentsLayout::Bottom => RenderResult {
            pattern: (&*SurfacePattern::create(&cr.get_target())).clone(),
            width: components.iter().map(|render_result| render_result.width).sum(),
            height: max_height,
        },
        ComponentsLayout::Left | ComponentsLayout::Center | ComponentsLayout::Right => RenderResult {
            pattern: (&*SurfacePattern::create(&cr.get_target())).clone(),
            width: max_width,
            height: components.iter().map(|render_result| render_result.height).sum(),
        },
    }
}

fn render(value: Value) -> RenderResult {
    if let Some(value_inner) = value.try_downcast::<HoldValueInner>() {
        let inner_render_result = render(value_inner.inner.clone());
        let underline_render_result = render_underline(inner_render_result.width, 1.0, 0.0, 0.0);
        render_components(ComponentsLayout::Center, &[inner_render_result, underline_render_result])
    } else if let Some(value_inner) = value.try_downcast::<ReleaseValueInner>() {
        let inner_render_result = render(value_inner.inner.clone());
        let underline_render_result = render_underline(inner_render_result.width, 0.0, 1.0, 0.0);
        render_components(ComponentsLayout::Center, &[inner_render_result, underline_render_result])
    } else if let Some(value_inner) = value.try_downcast::<AssignmentValueInner>() {
        render_components(
            ComponentsLayout::Middle,
            &[render(value_inner.target.clone()), render_text("<-"), render(value_inner.source.clone())],
        )
    } else if let Some(value_inner) = value.try_downcast::<DereferenceValueInner>() {
        render_components(ComponentsLayout::Middle, &[render_text("*"), render(value_inner.inner.clone())])
    } else if let Some(value_inner) = value.try_downcast::<ExecutableSequenceValueInner>() {
        render_components(
            ComponentsLayout::Left,
            &std::iter::once(render_text("{"))
                .chain(
                    value_inner
                        .inner
                        .iter()
                        .map(|value| render_components(ComponentsLayout::Middle, &[render_empty(20.0, 0.0), render(value.clone())]))
                        .intersperse(render_empty(0.0, 3.0)),
                )
                .chain(std::iter::once(render_text("}")))
                .collect::<Vec<_>>(),
        )
    } else if let Some(value_inner) = value.try_downcast::<ExecutableFunctionValueInner>() {
        render_components(
            ComponentsLayout::Middle,
            &[render(value_inner.arguments.clone()), render_text("->"), render(value_inner.body.clone())],
        )
    } else if let Some(value_inner) = value.try_downcast::<FunctionApplicationValueInner>() {
        render_components(
            ComponentsLayout::Middle,
            &[render(value_inner.function.clone()), render(value_inner.arguments.clone())],
        )
    } else if let Some(value_inner) = value.try_downcast::<IntrinsicCallValueInner>() {
        render_components(
            ComponentsLayout::Middle,
            &[render(value_inner.intrinsic.clone()), render(value_inner.arguments.clone())],
        )
    } else if let Some(value_inner) = value.try_downcast::<TupleValueInner>() {
        render_components(
            ComponentsLayout::Middle,
            &std::iter::once(render_text("("))
                .chain(
                    value_inner
                        .inner
                        .iter()
                        .map(|value| render(value.clone()))
                        .intersperse(render_components(ComponentsLayout::Bottom, &[render_text(","), render_empty(5.0, 10.0)])),
                )
                .chain(std::iter::once(render_text(")")))
                .collect::<Vec<_>>(),
        )
    } else if let Some(_) = value.try_downcast::<NullValueInner>() {
        render_text("null")
    } else if let Some(value_inner) = value.try_downcast::<SymbolValueInner>() {
        render_text(&value_inner.name)
    } else if let Some(value_inner) = value.try_downcast::<FloatingPointNumberValueInner>() {
        render_text(&value_inner.inner.to_string())
    } else {
        unreachable!()
    }
}

pub fn run(value: Value) {
    let application = Application::new(None, Default::default()).unwrap();
    application.connect_activate(move |application| {
        let window = ApplicationWindow::new(application);
        window.set_default_size(1366, 768);
        window.set_position(gtk::WindowPosition::Center);
        let drawing_area = DrawingArea::new();
        let value = value.clone();
        drawing_area.connect_draw(move |_, cr| {
            cr.set_source(&render_rasterized(render_scaled(render(value.clone()), 1.3, 1.3)).pattern);
            cr.paint();
            Inhibit(false)
        });
        window.add(&drawing_area);
        window.show_all();
    });
    application.run(&[]);
}
