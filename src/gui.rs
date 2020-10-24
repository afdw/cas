use crate::{data::*, Value};
use cairo::{Context, Pattern};
use gio::prelude::*;
use gtk::{prelude::*, Application, ApplicationWindow, DrawingArea};
use itertools::Itertools;

#[derive(Clone)]
struct RenderResult {
    pattern: Pattern,
    width: f64,
    height: f64,
}

fn render_empty(cr: &Context, width: f64, height: f64) -> RenderResult {
    cr.push_group();
    RenderResult {
        pattern: cr.pop_group(),
        width,
        height,
    }
}

fn render_text(cr: &Context, text: &str) -> RenderResult {
    cr.push_group();
    let text_extents = cr.text_extents(text);
    cr.save();
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.move_to(-text_extents.x_bearing, -text_extents.y_bearing);
    cr.show_text(text);
    cr.restore();
    RenderResult {
        pattern: cr.pop_group(),
        width: text_extents.width,
        height: text_extents.height,
    }
}

fn render_underline(cr: &Context, width: f64) -> RenderResult {
    cr.push_group();
    cr.set_line_width(1.0);
    cr.move_to(0.0, 1.5);
    cr.line_to(width, 1.0);
    cr.stroke();
    RenderResult {
        pattern: cr.pop_group(),
        width,
        height: 2.0,
    }
}

fn render_horizontal_components(cr: &Context, components: &[RenderResult]) -> RenderResult {
    cr.push_group();
    cr.save();
    cr.translate(
        0.0,
        components.iter().map(|render_result| render_result.height).fold_first(f64::max).unwrap_or(0.0) / 2.0,
    );
    for render_result in components {
        cr.save();
        cr.translate(0.0, -render_result.height / 2.0);
        cr.set_source(&render_result.pattern);
        cr.paint();
        cr.restore();
        cr.translate(render_result.width, 0.0);
    }
    cr.restore();
    RenderResult {
        pattern: cr.pop_group(),
        width: components.iter().map(|render_result| render_result.width).sum(),
        height: components.iter().map(|render_result| render_result.height).fold_first(f64::max).unwrap_or(0.0),
    }
}

fn render_vertical_components(cr: &Context, components: &[RenderResult]) -> RenderResult {
    cr.push_group();
    cr.save();
    // cr.translate(
    //     components.iter().map(|render_result|
    // render_result.width).fold_first(f64::max).unwrap_or(0.0) / 2.0,     0.0,
    // );
    for render_result in components {
        cr.save();
        // cr.translate(-render_result.width / 2.0, 0.0);
        cr.set_source(&render_result.pattern);
        cr.paint();
        cr.restore();
        cr.translate(0.0, render_result.height);
    }
    cr.restore();
    RenderResult {
        pattern: cr.pop_group(),
        width: components.iter().map(|render_result| render_result.width).fold_first(f64::max).unwrap_or(0.0),
        height: components.iter().map(|render_result| render_result.height).sum(),
    }
}

fn render(cr: &Context, value: Value) -> RenderResult {
    if let Some(value_inner) = value.try_downcast::<HoldValueInner>() {
        let inner_render_result = render(cr, value_inner.inner.clone());
        cr.set_source_rgb(1.0, 0.0, 0.0);
        let underline_render_result = render_underline(cr, inner_render_result.width);
        render_vertical_components(cr, &[inner_render_result, underline_render_result])
    } else if let Some(value_inner) = value.try_downcast::<ReleaseValueInner>() {
        let inner_render_result = render(cr, value_inner.inner.clone());
        cr.set_source_rgb(0.0, 1.0, 0.0);
        let underline_render_result = render_underline(cr, inner_render_result.width);
        render_vertical_components(cr, &[inner_render_result, underline_render_result])
    } else if let Some(value_inner) = value.try_downcast::<AssignmentValueInner>() {
        render_horizontal_components(
            cr,
            &[
                render(cr, value_inner.target.clone()),
                render_text(cr, "<-"),
                render(cr, value_inner.source.clone()),
            ],
        )
    } else if let Some(value_inner) = value.try_downcast::<DereferenceValueInner>() {
        render_horizontal_components(cr, &[render_text(cr, "*"), render(cr, value_inner.inner.clone())])
    } else if let Some(value_inner) = value.try_downcast::<ExecutableSequenceValueInner>() {
        render_vertical_components(
            cr,
            &std::iter::once(render_text(cr, "{"))
                .chain(
                    value_inner
                        .inner
                        .iter()
                        .map(|value| render_horizontal_components(cr, &[render_empty(cr, 20.0, 0.0), render(cr, value.clone())])),
                )
                .chain(std::iter::once(render_text(cr, "}")))
                .collect::<Vec<_>>(),
        )
    } else if let Some(value_inner) = value.try_downcast::<ExecutableFunctionValueInner>() {
        render_horizontal_components(
            cr,
            &[
                render(cr, value_inner.arguments.clone()),
                render_text(cr, "->"),
                render(cr, value_inner.body.clone()),
            ],
        )
    } else if let Some(value_inner) = value.try_downcast::<FunctionApplicationValueInner>() {
        render_horizontal_components(cr, &[render(cr, value_inner.function.clone()), render(cr, value_inner.arguments.clone())])
    } else if let Some(value_inner) = value.try_downcast::<IntrinsicCallValueInner>() {
        render_horizontal_components(cr, &[render(cr, value_inner.intrinsic.clone()), render(cr, value_inner.arguments.clone())])
    } else if let Some(value_inner) = value.try_downcast::<TupleValueInner>() {
        render_horizontal_components(
            cr,
            &std::iter::once(render_text(cr, "("))
                .chain(
                    value_inner
                        .inner
                        .iter()
                        .map(|value| render(cr, value.clone()))
                        .intersperse(render_horizontal_components(cr, &[render_text(cr, ","), render_empty(cr, 5.0, 0.0)])),
                )
                .chain(std::iter::once(render_text(cr, ")")))
                .collect::<Vec<_>>(),
        )
    } else if let Some(_) = value.try_downcast::<NullValueInner>() {
        render_text(cr, "null")
    } else if let Some(value_inner) = value.try_downcast::<SymbolValueInner>() {
        render_text(cr, &value_inner.name)
    } else if let Some(value_inner) = value.try_downcast::<FloatingPointNumberValueInner>() {
        render_text(cr, &value_inner.inner.to_string())
    } else {
        unreachable!()
    }
}

pub fn run(value: Value) {
    let application = Application::new(None, Default::default()).unwrap();
    application.connect_activate(move |application| {
        let window = ApplicationWindow::new(application);
        window.set_default_size(1280, 720);
        window.set_position(gtk::WindowPosition::Center);
        let drawing_area = DrawingArea::new();
        let value = value.clone();
        drawing_area.connect_draw(move |_, cr| {
            cr.scale(1.0, 1.0);
            let render_result = render(cr, value.clone());
            cr.set_source(&render_result.pattern);
            cr.paint();
            Inhibit(false)
        });
        window.add(&drawing_area);
        window.show_all();
    });
    application.run(&[]);
}
