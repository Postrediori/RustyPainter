use image::RgbImage;

use fltk::{
    app,
    dialog,
    draw::{capture_surface, draw_line, draw_point, draw_rect_fill, set_draw_color, set_line_style, LineStyle},
    enums::{Color, Event, FrameType, Shortcut},
    frame::Frame,
    menu,
    prelude::*,
    surface::ImageSurface,
    window::Window,
};
use std::cell::RefCell;
use std::rc::Rc;

struct Canvas {
    frame: Frame,
    #[allow(dead_code)]
    surf: Rc<RefCell<ImageSurface>>,
}

impl Canvas {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        let mut frame = Frame::default().with_size(w, h).with_pos(x, y);
        frame.set_color(Color::White);
        frame.set_frame(FrameType::DownBox);

        let surf = ImageSurface::new(frame.width(), frame.height(), false);

        let surf = Rc::from(RefCell::from(surf));

        frame.draw({
            let surf = surf.clone();
            move |f| {
                let surf = surf.borrow();
                let mut img = surf.image().unwrap();
                img.draw(f.x(), f.y(), f.w(), f.h());
            }
        });

        frame.handle({
            let mut x = 0;
            let mut y = 0;
            let surf = surf.clone();
            move |f, ev| {
                // println!("{}", ev);
                // println!("coords {:?}", app::event_coords());
                // println!("get mouse {:?}", app::get_mouse());
                let surf = surf.borrow_mut();
                match ev {
                    Event::Push => {
                        ImageSurface::push_current(&surf);
                        set_draw_color(Color::Red);
                        set_line_style(LineStyle::Solid, 3);
                        let coords = app::event_coords();
                        x = coords.0 - f.x();
                        y = coords.1 - f.y();
                        draw_point(x, y);
                        ImageSurface::pop_current();
                        f.redraw();
                        true
                    }
                    Event::Drag => {
                        ImageSurface::push_current(&surf);
                        set_draw_color(Color::Red);
                        set_line_style(LineStyle::Solid, 3);
                        let coords = app::event_coords();
                        let x2 = coords.0 - f.x();
                        let y2 = coords.1 - f.y();
                        draw_line(x, y, x2, y2);
                        x = x2;
                        y = y2;
                        ImageSurface::pop_current();
                        f.redraw();
                        true
                    }
                    _ => false,
                }
            }
        });
        Self { frame, surf }
    }

    pub fn clean_canvas(&self) {
        ImageSurface::push_current(&self.surf.borrow());
        draw_rect_fill(0, 0, self.frame.w(), self.frame.h(), Color::White);
        ImageSurface::pop_current();
    }

    /// Saves a canvas into an image file
    /// # Errors
    /// Errors on failure to save file
    pub fn save_image<P: AsRef<std::path::Path>>(&self, path: P) {
        assert!(!self.surf.as_ptr().is_null());

        let path = path
            .as_ref()
            .to_str().unwrap();

        let result = capture_surface(&self.surf.borrow(), self.frame.width(), self.frame.height());

        match result {
            Ok(img) => {
                let data = img.to_rgb_data();

                let img = RgbImage::from_raw(self.frame.width() as u32, self.frame.height() as u32, data.to_vec())
                    .expect("container should have the right size for the image dimensions");
            
                let _ = img.save(path);
            }
            
            Err(error) => { println!("Cannot save screenshot. Error: {}", error); }
        }
    }
}

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;

fltk::widget_extends!(Canvas, Frame, frame);

pub fn center() -> (i32, i32) {
    (
        (app::screen_size().0 / 2.0) as i32,
        (app::screen_size().1 / 2.0) as i32,
    )
}

fn main() {
    let app = app::App::default().with_scheme(app::Scheme::Gtk);

    let mut wind = Window::default()
        .with_size(WIDTH, HEIGHT)
        .with_label("RustyPainter");

    #[derive(Copy, Clone)]
    pub enum Message {
        New,
        Save,
        Quit,
    }

    let (tx, rx) = app::channel::<Message>();

    let mut menubar = menu::MenuBar::new(0, 0, WIDTH, 40, "rew");
    menubar.add_emit("&File/New\t", Shortcut::Ctrl | 'n', menu::MenuFlag::Normal, tx, Message::New);
    menubar.add_emit("&File/Save\t", Shortcut::Ctrl | 's', menu::MenuFlag::MenuDivider, tx, Message::Save);
    menubar.add_emit("&File/Quit\t", Shortcut::Ctrl | 'q', menu::MenuFlag::Normal, tx, Message::Quit);

    let mut canvas = Canvas::new(5, 45, WIDTH - 10, HEIGHT - 50);
    canvas.clean_canvas();

    wind.end();
    wind.show();

    app::add_idle3(move |_| {
        if let Some(msg) = rx.recv() {
            match msg {
            Message::New => { canvas.clean_canvas(); canvas.redraw(); }
            Message::Save => {
                let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseSaveFile);
                dlg.set_option(dialog::FileDialogOptions::SaveAsConfirm);
                dlg.show();
                if !dlg.filename().to_string_lossy().to_string().is_empty() {
                    let result = canvas.save_image(&dlg.filename());
                }
            }
            Message::Quit => { app.quit(); }
            }
        }
    });

    app.run().unwrap();
}
