use std::path::Path;
use std::borrow::Cow;
use iced::window::Position;
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
    Theme,
    alignment::Horizontal
};

// Widgets
use iced::widget::{Column, Text, Button, Row, TextInput, Container, Svg};

// Svg
use iced::widget::svg;

mod gd_path;
mod register;
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
    path_val: String,
    output_val: Result<String, String>,
    page: MessageType
}

impl Sandbox for What {
    type Message = MessageType;

    fn theme(&self) -> Theme {
        let mut palette = Theme::default().palette();
        palette.background = Color::new(0.1, 0.1, 0.1, 1.0);

        Theme::custom(palette)
    }

    fn title(&self) -> String {
        String::from("Geode Installer")
    }

    fn new() -> What {
        What {
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
                }
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        // Vertical alignment
        let col = Column::new()
            .align_items(Alignment::Center)
            .width(Length::Fill)
            .padding(Padding::from([30, 0, 50, 0]))
            .push(
                Svg::new(svg::Handle::from_memory(
                    Cow::from(&std::include_bytes!("../assets/geode-logo.svg")[..])
                )).height(Length::Units(75))
            )
            .push(
                Text::new("Install Geode")
                    .style(Color::new(0.93, 0.93, 0.93, 1.0)
            ).size(30));

        match self.page {
            MessageType::Main => {
                let path_input = TextInput::new(
                    "Path to Geometry Dash",
                    &self.path_val,
                    MessageType::PathChange
                ).style(MyInputStyle::theme())
                 .width(Length::Fill)
                 .size(18)
                 .padding(Padding::from([6, 8]));

                let path_select = Button::new(
                    Svg::new(svg::Handle::from_memory(
                        Cow::from(&std::include_bytes!("../assets/folder.svg")[..])
                    )).height(Length::Units(18))
                ).style(MyOtherBtnStyle::theme())
                 .padding(Padding::from([6, 10]))
                 .on_press(MessageType::ChoosePath);

                // Row for path selection and button
                let row1 = Row::new()
                    .padding(Padding::from([30, 30, 10, 30]))
                    .spacing(20)
                    .align_items(Alignment::Center)
                    .push(path_input)
                    .push(path_select);

                let mut install_btn = Button::new(
                    Text::new("Install").size(20).horizontal_alignment(Horizontal::Center)
                )
                    .style(MyBtnStyle::theme())
                    .padding(Padding::from([5, 10]))
                    .width(Length::Units(110));
                
                let mut uninstall_btn = Button::new(
                    Text::new("Uninstall").size(20).horizontal_alignment(Horizontal::Center)
                )
                    .style(MyBtnStyle::theme())
                    .padding(Padding::from([5, 10]))
                    .width(Length::Units(110));

                let mut err_text = Text::new("Enter a valid path to Geometry Dash.").size(16);
                if gd_path::validate_path(Path::new(&self.path_val)) {
                    err_text = err_text.style(Color::new(0.0, 0.0, 0.0, 0.0));

                    install_btn = install_btn.on_press(MessageType::Install);
                    uninstall_btn = uninstall_btn.on_press(MessageType::Uninstall);
                } else {
                    err_text = err_text.style(Color::new(1.0, 0.1, 0.1, 1.0));
                }

                // Row for the install/uninstall buttons
                let row2 = Row::new()
                    .height(Length::Fill)
                    .align_items(Alignment::Center)
                    .spacing(48)
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

                let return_btn = Button::new(
                    Text::new("Back").size(20)
                ).style(MyBtnStyle::theme())
                 .padding(Padding::from([5, 10]))
                 .on_press(MessageType::Main);

                col
                    .push(Text::new(text).style(color))
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
    settings.window.size = (400, 330);
    settings.window.min_size = Some((400, 330));
    settings.window.position = Position::Centered;

    What::run(settings)
}
