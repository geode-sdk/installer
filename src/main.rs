use std::path::Path;
use native_dialog::FileDialog;
use iced::Settings;
use iced::Sandbox;
use iced::Element;

// Style Utils
use iced::{
    Alignment,
    Length,
    Padding,
    Color,
    alignment::Horizontal
};

// Action Utils
use {
    iced::button::State as ButtonState,
    iced::text_input::State as TextInputState
};

// Widgets
use iced::widget::{Column, Text, Button, Row, TextInput, Container};

mod gd_path;
mod installation;
mod error;
mod styles;
use crate::styles::*;

#[derive(Clone, Debug)]
enum MessageType {
    Main,
    Install,
    Uninstall,
    PathChange(String),
    ChoosePath
}

struct What {
    install_btn: ButtonState,
    uninstall_btn: ButtonState,
    path_btn: ButtonState,
    back_btn: ButtonState,
    path_input: TextInputState,
    path_val: String,
    output_val: Result<String, String>,
    page: MessageType
}

impl Sandbox for What {
    type Message = MessageType;

    fn background_color(&self) -> Color {
        Color::new(0.1, 0.1, 0.1, 1.0)
    }

    fn title(&self) -> String {
        String::from("Geode Installer")
    }

    fn new() -> What {
        What {
            install_btn: ButtonState::default(),
            uninstall_btn: ButtonState::default(),
            path_btn: ButtonState::default(),
            back_btn: ButtonState::default(),
            path_input: TextInputState::default(),
            path_val: gd_path::find_path().unwrap_or(String::new()),
            output_val: Ok(String::new()),
            page: MessageType::Main
        }
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            MessageType::Main => {
                self.page = message;
            },
            MessageType::Install => {
                self.output_val = installation::install_to(Path::new(&self.path_val))
                    .map(|_| "Successfully installed Geode!".to_string());
                self.page = message;
            },
            MessageType::Uninstall => {
                self.output_val = installation::uninstall_from(Path::new(&self.path_val))
                    .map(|_| "Successfully uninstalled Geode!".to_string());
                self.page = message;
            },
            MessageType::PathChange(s) => {
                self.path_val = s;
            }
            MessageType::ChoosePath => {
                let mut dialog = FileDialog::new().set_location("~");

                if cfg!(target_os = "macos") {
                    dialog = dialog.add_filter("Application", &["app"]);
                } else if cfg!(windows) {
                    dialog = dialog.add_filter("Executable", &["exe"]);
                }

                if let Some(path) = dialog.show_open_single_file().unwrap() {
                    self.path_val = path.to_str().unwrap().to_string();
                    self.path_input.move_cursor_to_end();
                }
            }
        }
    }

    fn view(&mut self) -> Element<Self::Message> {
        let col = Column::new()
            .align_items(Alignment::Center)
            .width(Length::Fill)
            .padding(Padding::from([10, 0]))
            .push(Text::new("Geode Installer").color(Color::new(0.93, 0.93, 0.93, 1.0)).size(42));

        match self.page {
            MessageType::Main => {
                let path_input = TextInput::new(
                    &mut self.path_input,
                    "Path to Geometry Dash",
                    &self.path_val,
                    MessageType::PathChange
                ).style(MyInputStyle)
                 .width(Length::Fill)
                 .size(12)
                 .padding(Padding::from([6, 8]));

                let path_select = Button::new(&mut self.path_btn, Text::new("Browse...").size(16))
                    .style(MyOtherBtnStyle)
                    .padding(Padding::from([6, 15]))
                    .on_press(MessageType::ChoosePath);

                let row1 = Row::new()
                    .padding(Padding::from([30, 30, 10, 30]))
                    .spacing(20)
                    .align_items(Alignment::Center)
                    .push(path_input)
                    .push(path_select);

                let mut install_btn = Button::new(&mut self.install_btn, Text::new("Install").size(25).horizontal_alignment(Horizontal::Center))
                    .style(MyBtnStyle)
                    .width(Length::Units(120));
                let mut uninstall_btn = Button::new(&mut self.uninstall_btn, Text::new("Uninstall").size(25).horizontal_alignment(Horizontal::Center))
                    .style(MyBtnStyle)
                    .width(Length::Units(120));

                let mut err_text = Text::new("Enter a valid path to Geometry Dash.").size(16);
                if gd_path::validate_path(Path::new(&self.path_val)) {
                    err_text = err_text.color(Color::new(0.0, 0.0, 0.0, 0.0));

                    install_btn = install_btn.on_press(MessageType::Install);
                    uninstall_btn = uninstall_btn.on_press(MessageType::Uninstall);
                } else {
                    err_text = err_text.color(Color::new(1.0, 0.1, 0.1, 1.0));
                }

                let row2 = Row::new()
                    .height(Length::Fill)
                    .align_items(Alignment::Center)
                    .spacing(50)
                    .push(install_btn)
                    .push(uninstall_btn);

                col.push(row1).push(err_text).push(row2).into()
            },
            MessageType::Install | MessageType::Uninstall => {
                let text = match &self.output_val {
                    Ok(a) => a,
                    Err(b) => b
                };

                let color = if self.output_val.is_ok() {
                    Color::new(0.1, 1.0, 0.1, 1.0)
                } else {
                    Color::new(1.0, 0.1, 0.1, 1.0)
                };

                let return_btn = Button::new(&mut self.back_btn, Text::new("Back").size(25))
                    .style(MyBtnStyle)
                    .padding(Padding::from([6, 18]))
                    .on_press(MessageType::Main);

                col
                    .push(Text::new(text).color(color))
                    .spacing(20)
                    .push(Container::new(return_btn).height(Length::Fill).center_y().center_x())
                    .into()
            },
            _ => unreachable!()
        }
    }

}

pub fn main() -> iced::Result {
    let mut settings = Settings::default();
    settings.window.size = (400, 300);
    settings.window.min_size = Some((400, 300));

    println!("{:?}", settings.window.min_size);
    What::run(settings)
}