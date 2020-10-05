mod data;
mod value;

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
