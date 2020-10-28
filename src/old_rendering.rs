/*
use gio::prelude::*;
use gtk::{prelude::*, Application, ApplicationWindow, DrawingArea};
pub use value::Value;

fn main() {
    let application = Application::new(None, Default::default()).unwrap();
    application.connect_activate(|application| {
        let window = ApplicationWindow::new(application);
        let drawing_area = DrawingArea::new();
        drawing_area.connect_draw(|_, cr| {
            let value = Value::new(data::AdditionValueInner {
                a: Value::new(data::NumberLiteralValueInner { value: 0.3 }),
                b: Value::new(data::PlaceholderValueInner),
            });
            cr.scale(2.0, 2.0);
            let (pattern, (_width, _height)) = data::render(value, cr);
            cr.set_source(&pattern);
            cr.paint();
            Inhibit(false)
        });
        window.add(&drawing_area);
        window.show_all();
    });
    application.run(&[]);
}

pub struct PlaceholderValueInner;

impl PlaceholderValueInner {
    fn render(_value: Value, cr: &Context) -> (Pattern, (f64, f64)) {
        cr.push_group();
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.set_line_width(1.0);
        cr.move_to(0.5, 0.5);
        cr.line_to(6.5, 0.5);
        cr.line_to(6.5, 6.5);
        cr.line_to(0.5, 6.5);
        cr.close_path();
        cr.stroke();
        (cr.pop_group(), (7.0, 7.0))
    }
}

impl NumberLiteralValueInner {
    fn render(value: Value, cr: &Context) -> (Pattern, (f64, f64)) {
        let this = value.downcast::<Self>();
        cr.push_group();
        let text = this.value.to_string();
        let text_extents = cr.text_extents(&text);
        cr.move_to(-text_extents.x_bearing, -text_extents.y_bearing);
        cr.show_text(&text);
        (cr.pop_group(), (text_extents.width, text_extents.height))
    }
}

impl AdditionValueInner {
    fn render(value: Value, cr: &Context) -> (Pattern, (f64, f64)) {
        let this = value.downcast::<Self>();
        cr.push_group();
        let (a_pattern, (a_width, a_height)) = render(this.a.clone(), cr);
        let (b_pattern, (b_width, b_height)) = render(this.b.clone(), cr);
        {
            cr.save();
            cr.translate(0.0, f64::max(a_height, b_height) / 2.0);

            {
                cr.save();
                cr.translate(0.0, -a_height / 2.0);

                cr.set_source(&a_pattern);
                cr.paint();

                cr.restore();
            }
            {
                cr.save();
                cr.translate(a_width + 5.0, 0.0);

                cr.set_source_rgb(0.0, 0.0, 0.0);
                cr.set_line_width(1.0);
                cr.move_to(-3.0, 0.0);
                cr.line_to(3.0, 0.0);
                cr.move_to(0.0, -3.0);
                cr.line_to(0.0, 3.0);
                cr.stroke();

                cr.restore();
            }
            {
                cr.save();
                cr.translate(a_width + 10.0, -b_height / 2.0);

                cr.set_source(&b_pattern);
                cr.paint();

                cr.restore();
            }

            cr.restore();
        }
        (cr.pop_group(), (a_width + b_width + 10.0, f64::max(a_height, b_height)))
    }
}

pub fn render(value: Value, cr: &Context) -> (Pattern, (f64, f64)) {
    match value {
        value if value.is::<PlaceholderValueInner>() => PlaceholderValueInner::render(value, cr),
        value if value.is::<NumberLiteralValueInner>() => NumberLiteralValueInner::render(value, cr),
        value if value.is::<AdditionValueInner>() => AdditionValueInner::render(value, cr),
        _ => unreachable!(),
    }
}

impl SymbolValueInner {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(name: String) -> Value {
        Value::new(SymbolValueInner { name })
    }
}

fn render_frame(render_result: RenderResult) -> RenderResult {
    let cr = Context::new(&*RecordingSurface::create(Content::ColorAlpha, None).unwrap());
    cr.set_line_width(1.0);
    cr.rectangle(0.5, 0.5, render_result.width + 1.0, render_result.height + 1.0);
    cr.stroke();
    cr.translate(1.0, 1.0);
    cr.set_source(&render_result.pattern);
    cr.paint();
    RenderResult {
        pattern: (&*SurfacePattern::create(&cr.get_target())).clone(),
        width: render_result.width + 2.0,
        height: render_result.height + 2.0,
    }
}
*/
