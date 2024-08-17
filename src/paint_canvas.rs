use ::image::RgbImage;

use fltk::{prelude::*, *};
use std::cell::RefCell;
use std::rc::Rc;

type CoordOption = Option<draw::Coord<i32>>;

struct CanvasInternal {
    size: (i32, i32),
    fg_color: enums::Color,
    bg_color: enums::Color,
    instrument_size: i32,
    coord: CoordOption,
    coord_change_cb: Box<dyn FnMut(CoordOption)>,
}

impl CanvasInternal {
    fn new(w: i32, h: i32) -> Self {
        Self {
            size: (w, h),
            fg_color: enums::Color::Red,
            bg_color: enums::Color::White,
            instrument_size: 5,
            coord: None,
            coord_change_cb: Box::new(|_| {}),
        }
    }
    fn instrument_push(&mut self, coord: draw::Coord<i32>, surf: &surface::ImageSurface) {
        surface::ImageSurface::push_current(&surf);

        // Draw with current instrument
        draw::draw_circle_fill(coord.0, coord.1, self.instrument_size, self.fg_color);

        self.coord = Some(coord);
        (self.coord_change_cb.as_mut())(self.coord);

        surface::ImageSurface::pop_current();
    }
    fn instrument_drag(&mut self, coord_new: draw::Coord<i32>, surf: &surface::ImageSurface) {
        surface::ImageSurface::push_current(&surf);

        if let Some(c) = self.coord {
            draw::set_draw_color(self.fg_color);
            draw::set_line_style(
                draw::LineStyle::Solid | draw::LineStyle::CapRound,
                self.instrument_size,
            );
            draw::draw_line(c.0, c.1, coord_new.0, coord_new.1);

            self.coord = Some(coord_new);
            (self.coord_change_cb.as_mut())(self.coord);
        }

        surface::ImageSurface::pop_current();
    }
    fn instrument_released(&mut self, _coord: draw::Coord<i32>, _surf: &surface::ImageSurface) {
        //
    }
    fn instrument_move(&mut self, coord: draw::Coord<i32>, _surf: &surface::ImageSurface) {
        self.coord = Some(coord);
        (self.coord_change_cb.as_mut())(self.coord);
    }
    fn instrument_enter(&mut self, coord: draw::Coord<i32>) {
        self.coord = Some(coord);
        (self.coord_change_cb.as_mut())(self.coord);
    }
    fn instrument_leave(&mut self, _coord: draw::Coord<i32>) {
        self.coord = None;
        (self.coord_change_cb.as_mut())(self.coord);
    }
    fn draw(&self, x: i32, y: i32, w: i32, h: i32, surf: &surface::ImageSurface) {
        draw::push_clip(x, y, w, h);

        let mut img = surf.image().unwrap();
        img.draw(x, y, self.size.0, self.size.1);

        if let Some(c) = self.coord {
            let instrument_color = enums::Color::contrast(self.fg_color, self.bg_color);
            draw::set_draw_color(instrument_color);
            draw::set_line_style(draw::LineStyle::Solid, 1);
            draw::draw_circle(
                (x + c.0) as f64,
                (y + c.1) as f64,
                (self.instrument_size as f64) / 2.0,
            );
        }

        draw::pop_clip();
    }
    fn clean(&mut self, surf: &surface::ImageSurface) {
        surface::ImageSurface::push_current(&surf);

        draw::draw_rect_fill(0, 0, self.size.0, self.size.1, self.bg_color);

        surface::ImageSurface::pop_current();
    }
    fn coord_change<F: FnMut(CoordOption) + 'static>(&mut self, cb: F) {
        self.coord_change_cb = Box::new(cb);
    }
    fn get_fg_color(&self) -> (u8, u8, u8) {
        self.fg_color.to_rgb()
    }
    fn set_fg_color(&mut self, c: (u8, u8, u8)) {
        self.fg_color = enums::Color::from_rgb(c.0, c.1, c.2);
    }
    fn get_bg_color(&self) -> (u8, u8, u8) {
        self.bg_color.to_rgb()
    }
    fn set_bg_color(&mut self, c: (u8, u8, u8)) {
        self.bg_color = enums::Color::from_rgb(c.0, c.1, c.2);
    }
}

pub struct Canvas {
    frame: frame::Frame,
    #[allow(dead_code)]
    surf: Rc<RefCell<surface::ImageSurface>>,
    canvas_internal: Rc<RefCell<CanvasInternal>>,
}

impl Canvas {
    pub fn new(x: i32, y: i32, w: i32, h: i32, surf_w: i32, surf_h: i32) -> Self {
        let mut frame = frame::Frame::default().with_size(w, h).with_pos(x, y);
        frame.set_color(enums::Color::Dark3);
        frame.set_frame(enums::FrameType::NoBox);

        let surf = surface::ImageSurface::new(surf_w, surf_h, false);
        let surf = Rc::from(RefCell::from(surf));

        let canvas_internal = CanvasInternal::new(surf_w, surf_h);
        let canvas_internal = Rc::from(RefCell::from(canvas_internal));

        frame.draw({
            let surf = surf.clone();
            let canvas_internal = canvas_internal.clone();
            move |f| {
                let surf = surf.borrow();
                let canvas_internal = canvas_internal.borrow();

                canvas_internal.draw(f.x(), f.y(), f.w(), f.h(), &surf);
            }
        });

        frame.handle({
            let surf = surf.clone();
            let canvas_internal = canvas_internal.clone();
            move |f, ev| {
                let surf = surf.borrow_mut();
                let mut canvas_internal = canvas_internal.borrow_mut();
                match ev {
                    enums::Event::Push => {
                        let coords = app::event_coords();
                        let coords = draw::Coord::<i32>(coords.0 - f.x(), coords.1 - f.y());

                        canvas_internal.instrument_push(coords, &surf);

                        f.redraw();
                        true
                    }
                    enums::Event::Drag => {
                        let coords = app::event_coords();
                        let coords = draw::Coord::<i32>(coords.0 - f.x(), coords.1 - f.y());

                        canvas_internal.instrument_drag(coords, &surf);

                        f.redraw();
                        true
                    }
                    enums::Event::Released => {
                        let coords = app::event_coords();
                        let coords = draw::Coord::<i32>(coords.0 - f.x(), coords.1 - f.y());

                        canvas_internal.instrument_released(coords, &surf);

                        f.redraw();
                        true
                    }
                    enums::Event::Move => {
                        let coords = app::event_coords();
                        let coords = draw::Coord::<i32>(coords.0 - f.x(), coords.1 - f.y());

                        canvas_internal.instrument_move(coords, &surf);

                        f.redraw();
                        true
                    }
                    enums::Event::Enter => {
                        let coords = app::event_coords();
                        let coords = draw::Coord::<i32>(coords.0 - f.x(), coords.1 - f.y());

                        canvas_internal.instrument_enter(coords);

                        f.redraw();
                        true
                    }
                    enums::Event::Leave => {
                        let coords = app::event_coords();
                        let coords = draw::Coord::<i32>(coords.0 - f.x(), coords.1 - f.y());

                        canvas_internal.instrument_leave(coords);

                        f.redraw();
                        true
                    }
                    _ => false,
                }
            }
        });
        Self {
            frame,
            surf,
            canvas_internal,
        }
    }

    pub fn clean_canvas(&self) {
        self.canvas_internal.borrow_mut().clean(&self.surf.borrow());
    }

    pub fn coord_change<F: FnMut(CoordOption) + 'static>(&mut self, cb: F) {
        self.canvas_internal.borrow_mut().coord_change(cb);
    }

    pub fn get_fg_color(&self) -> (u8, u8, u8) {
        self.canvas_internal.borrow().get_fg_color()
    }
    pub fn set_fg_color(&mut self, c: (u8, u8, u8)) {
        self.canvas_internal.borrow_mut().set_fg_color(c);
    }

    pub fn get_bg_color(&self) -> (u8, u8, u8) {
        self.canvas_internal.borrow().get_bg_color()
    }
    pub fn set_bg_color(&mut self, c: (u8, u8, u8)) {
        self.canvas_internal.borrow_mut().set_bg_color(c);
    }

    pub fn get_size(&self) -> (i32, i32) {
        self.canvas_internal.borrow().size
    }
    pub fn set_image_size(&mut self, size: (i32, i32)) {
        let old_size = self.canvas_internal.borrow_mut().size;
        self.canvas_internal.borrow_mut().size = size;

        let surf = surface::ImageSurface::new(size.0, size.1, false);
        let old_surf = self.surf.replace(surf);

        self.clean_canvas();

        // Draw old surface on top of the new one
        if let Ok(img) = draw::capture_surface(&old_surf, old_size.0, old_size.1) {
            surface::ImageSurface::push_current(&self.surf.borrow_mut());

            let data = img.to_rgb_data();

            let _ = draw::draw_image(
                &data.to_vec(),
                0,
                0,
                old_size.0,
                old_size.1,
                enums::ColorDepth::Rgb8,
            );

            surface::ImageSurface::pop_current();
        }

        self.set_size(size.0, size.1);
        self.frame.set_size(size.0, size.1);
    }

    /// Saves a canvas into an image file
    /// # Errors
    /// Errors on failure to save file
    pub fn save_image<P: AsRef<std::path::Path>>(&self, path: P) -> bool {
        // assert!(!self.surf.as_ptr().is_null());

        let path = path.as_ref().to_str().unwrap();

        let result =
            draw::capture_surface(&self.surf.borrow(), self.frame.width(), self.frame.height());

        match result {
            Ok(img) => {
                let data = img.to_rgb_data();

                let img = RgbImage::from_raw(
                    self.frame.width() as u32,
                    self.frame.height() as u32,
                    data.to_vec(),
                )
                .expect("container should have the right size for the image dimensions");

                let result = img.save(path);
                match result {
                    Ok(()) => {
                        println!("Save image to file {}", path);
                        true
                    }
                    Err(error) => {
                        eprintln!("Cannot save image to file {}. Error: {}", path, error);
                        false
                    }
                }
            }
            Err(error) => {
                eprintln!("Cannot save image to file {}. Error: {}", path, error);
                false
            }
        }
    }
}

fltk::widget_extends!(Canvas, frame::Frame, frame);
