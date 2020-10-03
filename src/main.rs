use cairo::{Context, Pattern};
use gio::prelude::*;
use gtk::{prelude::*, Application, ApplicationWindow, DrawingArea};

trait Value {
    fn render(&self, cr: &Context) -> (Pattern, (f64, f64));
}

struct Number {
    value: f64,
}

impl Value for Number {
    fn render(&self, cr: &Context) -> (Pattern, (f64, f64)) {
        cr.push_group();
        let text = self.value.to_string();
        let text_extents = cr.text_extents(&text);
        cr.move_to(-text_extents.x_bearing, -text_extents.y_bearing);
        cr.show_text(&text);
        let pattern = cr.pop_group();
        (pattern, (text_extents.width, text_extents.height))
    }
}

struct Addition {
    a: Box<dyn Value>,
    b: Box<dyn Value>,
}

impl Value for Addition {
    fn render(&self, cr: &Context) -> (Pattern, (f64, f64)) {
        cr.push_group();
        let (a_pattern, (a_width, a_height)) = self.a.render(cr);
        let (b_pattern, (b_width, b_height)) = self.b.render(cr);
        cr.set_source(&a_pattern);
        cr.paint();
        cr.translate(a_width, 0.0);
        cr.save();
        cr.translate(5.0, (a_height + b_height) / 4.0);
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.set_line_width(1.0);
        cr.move_to(-3.0, 0.0);
        cr.line_to(3.0, 0.0);
        cr.move_to(0.0, -3.0);
        cr.line_to(0.0, 3.0);
        cr.stroke();
        cr.restore();
        cr.translate(10.0, 0.0);
        cr.set_source(&b_pattern);
        cr.paint();
        let pattern = cr.pop_group();
        (pattern, (a_width + b_width, f64::max(a_height, b_height)))
    }
}

fn main() {
    let application = Application::new(None, Default::default()).unwrap();
    application.connect_activate(|application| {
        let window = ApplicationWindow::new(application);
        let drawing_area = DrawingArea::new();
        drawing_area.connect_draw(|_, cr| {
            let value = Addition {
                a: Box::new(Number { value: 0.3 }),
                b: Box::new(Number { value: 123.0 }),
            };
            cr.scale(2.0, 2.0);
            let (pattern, (_width, _height)) = value.render(cr);
            cr.set_source(&pattern);
            cr.paint();
            Inhibit(false)
        });
        window.add(&drawing_area);
        window.show_all();
    });
    application.run(&[]);
}
