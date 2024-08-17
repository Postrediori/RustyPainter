mod attributes_dialog;
mod paint_canvas;
mod res;

use fltk::{prelude::*, *};
use paint_canvas::Canvas;
use res::IconsAssets;

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
const MENUBAR_SIZE: i32 = 30;
const STATUSBAR_SIZE: i32 = 25;

const DEFAULT_CANVAS_WIDTH: i32 = 640;
const DEFAULT_CANVAS_HEIGHT: i32 = 480;

fn save_image(canvas: &Canvas, filename: &std::path::Path) {
    let result = canvas.save_image(&filename);
    let filename_str = filename.to_string_lossy().to_string();
    if result {
        println!("Saved image to file {}", filename_str);
    } else {
        eprintln!("Error while saving image to file {}", filename_str);
    }
}

fn save_image_as(canvas: &Canvas) -> std::path::PathBuf {
    const DEFAULT_FILENAME: &str = "untitled.bmp";
    let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseSaveFile);

    dlg.set_option(dialog::FileDialogOptions::SaveAsConfirm);
    dlg.set_filter("Bitmap\t*.bmp\nJPEG\t*.{jpg,jpeg}\nGIF\t*.gif\nTIFF\t*.{tif,tiff}\nPNG\t*.png");
    dlg.set_preset_file(DEFAULT_FILENAME);

    dlg.show();

    let filename = dlg.filename();
    if !filename.to_string_lossy().to_string().is_empty() {
        save_image(&canvas, &filename)
    } else {
        eprintln!("Unable to save an image, file name is empty");
    }

    filename
}

fn main() {
    let app = app::App::default().with_scheme(app::Scheme::Gtk);

    let mut set_size_dialog = attributes_dialog::AttributesDialog::new();

    let mut current_filename = std::path::PathBuf::new();

    let mut wind = window::Window::default()
        .with_size(WIDTH, HEIGHT)
        .with_label("Rusty Painter");

    #[derive(Copy, Clone)]
    pub enum Message {
        New,
        Save,
        SaveAs,
        Quit,
        SetImageSize,
        ClearImage,
        SetFgColor,
        SetBgColor,
        About,
    }

    let (tx, rx) = app::channel::<Message>();

    let mut main_layout = group::Flex::default_fill().column();
    main_layout.set_margin(0);

    // Menubar
    let mut menubar = menu::MenuBar::new(0, 0, WIDTH, MENUBAR_SIZE, "rew");
    menubar.add_emit(
        "&File/New\t",
        enums::Shortcut::Ctrl | 'n',
        menu::MenuFlag::Normal,
        tx,
        Message::New,
    );
    menubar.add_emit(
        "&File/Save\t",
        enums::Shortcut::Ctrl | 's',
        menu::MenuFlag::Normal,
        tx,
        Message::Save,
    );
    menubar.add_emit(
        "&File/Save As...\t",
        enums::Shortcut::Ctrl | enums::Shortcut::Shift | 's',
        menu::MenuFlag::MenuDivider,
        tx,
        Message::SaveAs,
    );
    menubar.add_emit(
        "&File/Quit\t",
        enums::Shortcut::Ctrl | 'q',
        menu::MenuFlag::Normal,
        tx,
        Message::Quit,
    );
    menubar.add_emit(
        "&Image/Attributes\t",
        enums::Shortcut::Ctrl | 'e',
        menu::MenuFlag::Normal,
        tx,
        Message::SetImageSize,
    );
    menubar.add_emit(
        "&Image/Clear Image\t",
        enums::Shortcut::Ctrl | enums::Shortcut::Shift | 'n',
        menu::MenuFlag::Normal,
        tx,
        Message::ClearImage,
    );
    menubar.add_emit(
        "&Colors/Foreground...\t",
        enums::Shortcut::None,
        menu::MenuFlag::Normal,
        tx,
        Message::SetFgColor,
    );
    menubar.add_emit(
        "&Colors/Background...\t",
        enums::Shortcut::None,
        menu::MenuFlag::Normal,
        tx,
        Message::SetBgColor,
    );
    menubar.add_emit(
        "&Help/About\t",
        enums::Shortcut::None,
        menu::MenuFlag::Normal,
        tx,
        Message::About,
    );

    menubar.set_frame(enums::FrameType::ThinUpBox);

    const ACTION_ICONS: &[(&str, &str)] = &[
        ("&File/New\t", "document-new.svg"),
        ("&File/Save\t", "document-save.svg"),
        ("&File/Save As...\t", "document-save-as.svg"),
        ("&File/Quit\t", "application-exit.svg"),
        ("&Image/Attributes\t", "configure.svg"),
        ("&Image/Clear Image\t", "fill-color.svg"),
        ("&Help/About\t", "help-about.svg"),
    ];
    for i in ACTION_ICONS {
        if let Some(mut ii) = menubar.find_item(i.0) {
            if let Some(img) = IconsAssets::get(i.1) {
                if let Ok(img) = std::str::from_utf8(img.data.as_ref()) {
                    if let Ok(mut img) = fltk::image::SvgImage::from_data(img) {
                        img.scale(24, 24, true, true);
                        ii.add_image(Some(img), true);
                    }
                }
            }
        }
    }

    main_layout.fixed(&mut menubar, MENUBAR_SIZE);

    // Drawing canvas
    let mut canvas_frame: group::Scroll;
    let mut canvas: Canvas;
    {
        canvas_frame = group::Scroll::default();
        canvas_frame.set_color(enums::Color::Dark3);
        canvas_frame.set_frame(enums::FrameType::DownBox);

        canvas = Canvas::new(
            0,
            0,
            DEFAULT_CANVAS_WIDTH,
            DEFAULT_CANVAS_HEIGHT,
            DEFAULT_CANVAS_WIDTH,
            DEFAULT_CANVAS_HEIGHT,
        );
        canvas.clean_canvas();

        canvas_frame.end();
    }

    // Statusbar
    let mut filename_status: frame::Frame;
    let mut current_coord_status: frame::Frame;
    {
        let mut status_bar = group::Flex::default_fill().row();
        status_bar.set_margin(1);

        // Statusbar section with file name
        filename_status = frame::Frame::default();
        filename_status.set_align(enums::Align::Left | enums::Align::Inside);
        filename_status.set_frame(enums::FrameType::DownBox);

        // Current coordinate
        current_coord_status = frame::Frame::default();
        current_coord_status.set_align(enums::Align::Left | enums::Align::Inside);
        current_coord_status.set_frame(enums::FrameType::DownBox);
        status_bar.fixed(&current_coord_status, 125);

        status_bar.end();
        main_layout.fixed(&mut status_bar, STATUSBAR_SIZE);
    }

    // Finish creating the main window
    main_layout.end();

    wind.make_resizable(true);
    wind.end();
    wind.show();

    canvas.coord_change({
        move |c| {
            let str = match c {
                Some(c) => {
                    format!("{},{}", c.0, c.1)
                }
                None => "".to_string(),
            };
            current_coord_status.set_label(&str);
        }
    });

    while app.wait() {
        if let Some(msg) = rx.recv() {
            match msg {
                Message::New => {
                    canvas.clean_canvas();
                    canvas.redraw();

                    current_filename = std::path::PathBuf::new();
                    filename_status.set_label(&current_filename.to_string_lossy().to_string());
                }
                Message::Save => {
                    // Save to current file name or save to new file if current file name is empty
                    if current_filename.to_string_lossy().to_string().is_empty() {
                        current_filename = save_image_as(&canvas);
                        filename_status.set_label(&current_filename.to_string_lossy().to_string());
                    } else {
                        save_image(&canvas, &current_filename);
                    }
                }
                Message::SaveAs => {
                    // Always save to new file
                    current_filename = save_image_as(&canvas);
                    filename_status.set_label(&current_filename.to_string_lossy().to_string());
                }
                Message::Quit => {
                    app.quit();
                }
                Message::SetImageSize => {
                    let current_size = canvas.get_size();
                    if let Some(new_size) = set_size_dialog.show(current_size) {
                        canvas.set_image_size(new_size);
                        canvas_frame.redraw();
                    }
                }
                Message::ClearImage => {
                    canvas.clean_canvas();
                    canvas_frame.redraw();
                }
                Message::SetFgColor => {
                    let current_fg_color = canvas.get_fg_color();
                    let fg_color = dialog::color_chooser_with_default(
                        "Select Foreground color",
                        dialog::ColorMode::Rgb,
                        current_fg_color,
                    );
                    canvas.set_fg_color(fg_color);
                }
                Message::SetBgColor => {
                    let current_bg_color = canvas.get_bg_color();
                    let bg_color = dialog::color_chooser_with_default(
                        "Select Background color",
                        dialog::ColorMode::Rgb,
                        current_bg_color,
                    );
                    canvas.set_bg_color(bg_color);
                }
                Message::About => {
                    fltk::app::lock().unwrap();

                    const VERSION: &str = env!("CARGO_PKG_VERSION");

                    fltk::dialog::message_title("About");
                    fltk::dialog::message_set_hotspot(true);
                    fltk::dialog::message_icon_label("i");

                    let str = format!("Rusty Painter v{}", &VERSION);
                    fltk::dialog::message_default(&str);

                    fltk::app::unlock();
                }
            }
        }
    }
}
