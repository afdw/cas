use crate::{data::*, Value};
use cairo::{Content, Context, Format, ImageSurface, Pattern, RecordingSurface, SurfacePattern};
use gio::prelude::*;
use gtk::{prelude::*, Application, ApplicationWindow, DrawingArea};
use itertools::Itertools;
use std::{cell::RefCell, rc::Rc};

fn get_parts(value: Value) -> Vec<Value> {
    if let Some(value_inner) = value.try_downcast::<HoldValueInner>() {
        vec![value_inner.inner.clone()]
    } else if let Some(value_inner) = value.try_downcast::<ReleaseValueInner>() {
        vec![value_inner.inner.clone()]
    } else if let Some(value_inner) = value.try_downcast::<AssignmentValueInner>() {
        vec![value_inner.source.clone(), value_inner.target.clone()]
    } else if let Some(value_inner) = value.try_downcast::<DereferenceValueInner>() {
        vec![value_inner.inner.clone()]
    } else if let Some(value_inner) = value.try_downcast::<ExecutableSequenceValueInner>() {
        value_inner.inner.clone()
    } else if let Some(value_inner) = value.try_downcast::<ExecutableFunctionValueInner>() {
        vec![value_inner.arguments.clone(), value_inner.body.clone()]
    } else if let Some(value_inner) = value.try_downcast::<FunctionApplicationValueInner>() {
        vec![value_inner.function.clone(), value_inner.arguments.clone()]
    } else if let Some(value_inner) = value.try_downcast::<IntrinsicCallValueInner>() {
        vec![value_inner.intrinsic.clone(), value_inner.arguments.clone()]
    } else if let Some(value_inner) = value.try_downcast::<TupleValueInner>() {
        value_inner.inner.clone()
    } else {
        vec![]
    }
}

fn replace_parts(value: Value, parts: &[Value]) -> Value {
    if let Some(value_inner) = value.try_downcast::<HoldValueInner>() {
        assert_eq!(parts.len(), 1);
        let inner = parts[0].clone();
        if inner == value_inner.inner {
            value
        } else {
            Value::new(HoldValueInner { inner })
        }
    } else if let Some(value_inner) = value.try_downcast::<ReleaseValueInner>() {
        assert_eq!(parts.len(), 1);
        let inner = parts[0].clone();
        if inner == value_inner.inner {
            value
        } else {
            Value::new(ReleaseValueInner { inner })
        }
    } else if let Some(value_inner) = value.try_downcast::<AssignmentValueInner>() {
        assert_eq!(parts.len(), 2);
        let source = parts[0].clone();
        let target = parts[1].clone();
        if source == value_inner.source && target == value_inner.target {
            value
        } else {
            Value::new(AssignmentValueInner { source, target })
        }
    } else if let Some(value_inner) = value.try_downcast::<DereferenceValueInner>() {
        assert_eq!(parts.len(), 1);
        let inner = parts[0].clone();
        if inner == value_inner.inner {
            value
        } else {
            Value::new(DereferenceValueInner { inner })
        }
    } else if let Some(value_inner) = value.try_downcast::<ExecutableSequenceValueInner>() {
        let inner = parts.to_vec();
        if inner == value_inner.inner {
            value
        } else {
            Value::new(ExecutableSequenceValueInner { inner })
        }
    } else if let Some(value_inner) = value.try_downcast::<ExecutableFunctionValueInner>() {
        assert_eq!(parts.len(), 2);
        let arguments = parts[0].clone();
        let body = parts[1].clone();
        if arguments == value_inner.arguments && body == value_inner.body {
            value
        } else {
            Value::new(ExecutableFunctionValueInner { arguments, body })
        }
    } else if let Some(value_inner) = value.try_downcast::<FunctionApplicationValueInner>() {
        assert_eq!(parts.len(), 2);
        let function = parts[0].clone();
        let arguments = parts[1].clone();
        if function == value_inner.function && arguments == value_inner.arguments {
            value
        } else {
            Value::new(FunctionApplicationValueInner { function, arguments })
        }
    } else if let Some(value_inner) = value.try_downcast::<IntrinsicCallValueInner>() {
        assert_eq!(parts.len(), 2);
        let intrinsic = parts[0].clone();
        let arguments = parts[1].clone();
        if intrinsic == value_inner.intrinsic && arguments == value_inner.arguments {
            value
        } else {
            Value::new(IntrinsicCallValueInner { intrinsic, arguments })
        }
    } else if let Some(value_inner) = value.try_downcast::<TupleValueInner>() {
        let inner = parts.to_vec();
        if inner == value_inner.inner {
            value
        } else {
            Value::new(TupleValueInner { inner })
        }
    } else {
        unreachable!()
    }
}

fn get_part(value: Value, path: &[usize]) -> Option<Value> {
    if path.is_empty() {
        Some(value)
    } else {
        get_part(get_parts(value).get(path[0])?.clone(), &path[1..])
    }
}

fn replace_part(value: Value, path: &[usize], replacement: Value) -> Value {
    if path.is_empty() {
        replacement
    } else {
        let mut parts = get_parts(value.clone());
        parts[path[0]] = replace_part(parts[path[0]].clone(), &path[1..], replacement);
        replace_parts(value, &parts)
    }
}

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

fn render_highlighted(render_result: RenderResult, red: f64, green: f64, blue: f64, alpha: f64) -> RenderResult {
    let cr = Context::new(&*RecordingSurface::create(Content::ColorAlpha, None).unwrap());
    cr.set_line_width(1.0);
    cr.set_source_rgba(red, green, blue, alpha);
    cr.rectangle(0.0, 0.0, render_result.width, render_result.height);
    cr.fill();
    cr.set_source(&render_result.pattern);
    cr.paint();
    RenderResult {
        pattern: (&*SurfacePattern::create(&cr.get_target())).clone(),
        width: render_result.width,
        height: render_result.height,
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

fn render_value<RenderPart: FnMut(usize) -> RenderResult>(value: Value, mut render_part: RenderPart) -> RenderResult {
    #[allow(clippy::if_same_then_else)]
    if value.is::<HoldValueInner>() {
        let inner_render_result = render_part(0);
        let underline_render_result = render_underline(inner_render_result.width, 1.0, 0.0, 0.0);
        render_components(ComponentsLayout::Center, &[inner_render_result, underline_render_result])
    } else if value.is::<ReleaseValueInner>() {
        let inner_render_result = render_part(0);
        let underline_render_result = render_underline(inner_render_result.width, 0.0, 1.0, 0.0);
        render_components(ComponentsLayout::Center, &[inner_render_result, underline_render_result])
    } else if value.is::<AssignmentValueInner>() {
        render_components(ComponentsLayout::Middle, &[render_part(1), render_text("<-"), render_part(0)])
    } else if value.is::<DereferenceValueInner>() {
        render_components(ComponentsLayout::Middle, &[render_text("*"), render_part(0)])
    } else if let Some(value_inner) = value.try_downcast::<ExecutableSequenceValueInner>() {
        render_components(
            ComponentsLayout::Left,
            &std::iter::once(render_text("{"))
                .chain(
                    (0..value_inner.inner.len())
                        .map(|part_index| render_components(ComponentsLayout::Middle, &[render_empty(20.0, 0.0), render_part(part_index)]))
                        .intersperse(render_empty(0.0, 3.0)),
                )
                .chain(std::iter::once(render_text("}")))
                .collect::<Vec<_>>(),
        )
    } else if value.is::<ExecutableFunctionValueInner>() {
        render_components(ComponentsLayout::Middle, &[render_part(0), render_text("->"), render_part(1)])
    } else if value.is::<FunctionApplicationValueInner>() {
        render_components(ComponentsLayout::Middle, &[render_part(0), render_part(1)])
    } else if value.is::<IntrinsicCallValueInner>() {
        render_components(ComponentsLayout::Middle, &[render_part(0), render_part(1)])
    } else if let Some(value_inner) = value.try_downcast::<TupleValueInner>() {
        render_components(
            ComponentsLayout::Middle,
            &std::iter::once(render_text("("))
                .chain(
                    (0..value_inner.inner.len())
                        .map(|part_index| render_part(part_index))
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

fn render(value: Value, selection: Option<&[usize]>) -> RenderResult {
    let render_result = render_value(value.clone(), |part_index| {
        render(
            get_parts(value.clone())[part_index].clone(),
            if selection.is_some() && !selection.unwrap().is_empty() && selection.unwrap()[0] == part_index {
                Some(&selection.unwrap()[1..])
            } else {
                None
            },
        )
    });
    if selection.is_some() && selection.unwrap().is_empty() {
        render_highlighted(render_result, 0.0, 0.0, 0.0, 0.2)
    } else {
        render_result
    }
}

pub fn run(value: Value) {
    let application = Application::new(None, Default::default()).unwrap();
    application.connect_activate(move |application| {
        let window = ApplicationWindow::new(application);
        window.set_default_size(1366, 768);
        window.set_position(gtk::WindowPosition::Center);
        let drawing_area = DrawingArea::new();
        drawing_area.set_can_focus(true);
        let value = Rc::new(RefCell::new(value.clone()));
        let selection = Rc::new(RefCell::new(vec![]));
        {
            let value = value.clone();
            let selection = selection.clone();
            drawing_area.connect_key_press_event(move |drawing_area, event| match event.get_keyval() {
                gdk::keys::constants::Down => {
                    let part = get_part(value.borrow().clone(), &selection.borrow());
                    if part.is_some() && !get_parts(part.unwrap()).is_empty() {
                        selection.borrow_mut().push(0);
                        drawing_area.queue_draw();
                    }
                    Inhibit(true)
                }
                gdk::keys::constants::Up => {
                    selection.borrow_mut().pop();
                    drawing_area.queue_draw();
                    Inhibit(true)
                }
                gdk::keys::constants::Left => {
                    if !selection.borrow().is_empty() && *selection.borrow().last().unwrap() >= 1 {
                        *selection.borrow_mut().last_mut().unwrap() -= 1;
                        drawing_area.queue_draw();
                    }
                    Inhibit(true)
                }
                gdk::keys::constants::Right => {
                    if !selection.borrow().is_empty() {
                        let part = get_part(value.borrow().clone(), &selection.borrow()[..selection.borrow().len() - 1]);
                        if part.is_some() && selection.borrow().last().unwrap() + 1 < get_parts(part.unwrap()).len() {
                            *selection.borrow_mut().last_mut().unwrap() += 1;
                            drawing_area.queue_draw();
                        }
                    }
                    Inhibit(true)
                }
                _ => Inhibit(false),
            });
        }
        drawing_area.connect_draw(move |_, cr| {
            cr.set_source(&render_rasterized(render_scaled(render(value.borrow().clone(), Some(&selection.borrow())), 1.3, 1.3)).pattern);
            cr.paint();
            Inhibit(false)
        });
        window.add(&drawing_area);
        window.show_all();
    });
    application.run(&[]);
}
