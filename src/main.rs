use gio::prelude::*;
use gtk::{prelude::*, Application, ApplicationWindow, DrawingArea};

fn main() {
    let application = Application::new(None, Default::default()).unwrap();
    application.connect_activate(|application| {
        let window = ApplicationWindow::new(application);
        let drawing_area = DrawingArea::new();
        drawing_area.connect_draw(|_, cr| {
            cr.move_to(0.0, 10.0);
            cr.text_path("test");
            cr.fill();
            Inhibit(false)
        });
        window.add(&drawing_area);
        window.show_all();
    });
    application.run(&[]);
}
