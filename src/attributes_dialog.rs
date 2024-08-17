use fltk::{prelude::*, *};

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Copy, Clone)]
enum ModalResult {
    Ok,
    Cancel,
}

pub struct AttributesDialog {
    window: window::Window,
    width_input: input::IntInput,
    height_input: input::IntInput,
    modal_result: Rc<RefCell<ModalResult>>,
}

impl AttributesDialog {
    pub fn new() -> Self {
        let model_result = ModalResult::Cancel;
        let modal_result = Rc::from(RefCell::from(model_result));

        let mut window = window::Window::default()
            .with_label("Attributes")
            .with_size(350, 70);

        let width_input = input::IntInput::default()
            .with_label("Width: ")
            .with_size(75, 25)
            .with_pos(55, 15);

        let height_input = input::IntInput::default()
            .with_label("Height: ")
            .with_size(75, 25)
            .with_pos(185, 15);

        let mut ok_btn = button::Button::default()
            .with_label("&OK")
            .with_size(75, 25)
            .with_pos(265, 5);

        ok_btn.set_callback({
            let mut window = window.clone();
            let modal_result = modal_result.clone();
            move |_| {
                *modal_result.borrow_mut() = ModalResult::Ok;
                window.hide();
            }
        });

        let mut cancel_btn = button::Button::default()
            .with_label("&Cancel")
            .with_size(75, 25)
            .with_pos(265, 35);

        cancel_btn.set_callback({
            let mut window = window.clone();
            move |_| {
                window.hide();
            }
        });

        window.end();

        window.make_modal(true);

        Self {
            window,
            width_input,
            height_input,
            modal_result,
        }
    }

    pub fn show(&mut self, size: (i32, i32)) -> Option<(i32, i32)> {
        self.set_inputs(size);

        self.window.show();
        while self.window.shown() {
            app::wait();
        }

        match *self.modal_result.borrow() {
            ModalResult::Ok => Some(self.get_inputs()),
            ModalResult::Cancel => None,
        }
    }

    fn set_inputs(&mut self, size: (i32, i32)) {
        self.width_input.set_value(size.0.to_string().as_str());
        self.height_input.set_value(size.1.to_string().as_str());
    }

    fn get_inputs(&self) -> (i32, i32) {
        (
            self.width_input
                .value()
                .parse::<i32>()
                .expect("Not a number!"),
            self.height_input
                .value()
                .parse::<i32>()
                .expect("Not a number!"),
        )
    }
}
