use iced::{
	widget::{
		button::Appearance as ButtonAppearance, button::StyleSheet as ButtonSheet,
		text_input::Appearance as TextInputAppearance, text_input::StyleSheet as TextInputSheet,
	},
	Background, Color,
};

pub struct MyBtnStyle;
impl ButtonSheet for MyBtnStyle {
	type Style = iced::Theme;

	fn active(&self, _style: &Self::Style) -> ButtonAppearance {
		let mut s = ButtonAppearance::default();
		s.border_radius = 3.0;
		s.background = Some(Background::Color(Color::new(0.255, 0.24, 0.27, 1.0)));
		s.border_color = Color::new(0.73, 0.73, 0.73, 1.0);
		s.text_color = Color::new(0.93, 0.93, 0.93, 1.0);
		s
	}

	fn hovered(&self, style: &Self::Style) -> ButtonAppearance {
		let mut s = self.active(style);
		s.background = Some(Background::Color(Color::new(0.38, 0.35, 0.40, 1.0)));
		s
	}

	fn pressed(&self, style: &Self::Style) -> ButtonAppearance {
		self.active(style)
	}
}
impl MyBtnStyle {
	pub fn theme() -> iced::theme::Button {
		iced::theme::Button::Custom(Box::from(MyBtnStyle))
	}
}

pub struct MyOtherBtnStyle;
impl ButtonSheet for MyOtherBtnStyle {
	type Style = iced::Theme;

	fn active(&self, _style: &Self::Style) -> ButtonAppearance {
		let mut s = ButtonAppearance::default();
		s.border_radius = 3.0;
		s.background = Some(Background::Color(Color::new(0.24, 0.24, 0.24, 1.0)));
		s.border_color = Color::new(0.73, 0.73, 0.73, 1.0);
		s.text_color = Color::new(0.93, 0.93, 0.93, 1.0);
		s
	}

	fn hovered(&self, style: &Self::Style) -> ButtonAppearance {
		let mut s = self.active(style);
		s.background = Some(Background::Color(Color::new(0.35, 0.35, 0.35, 1.0)));
		s
	}

	fn pressed(&self, style: &Self::Style) -> ButtonAppearance {
		self.active(style)
	}
}
impl MyOtherBtnStyle {
	pub fn theme() -> iced::theme::Button {
		iced::theme::Button::Custom(Box::from(MyOtherBtnStyle))
	}
}

pub struct MyInputStyle;
impl TextInputSheet for MyInputStyle {
	type Style = iced::Theme;

	fn active(&self, _style: &Self::Style) -> TextInputAppearance {
		TextInputAppearance {
			background: Background::Color(Color::new(0.2, 0.2, 0.2, 1.0)),
			border_radius: 0.0,
			border_width: 0.0,
			border_color: Color::new(0.0, 0.0, 0.0, 0.0),
		}
	}
	fn focused(&self, style: &Self::Style) -> TextInputAppearance {
		self.active(style)
	}
	fn placeholder_color(&self, _style: &Self::Style) -> Color {
		Color::new(0.63, 0.63, 0.63, 1.0)
	}

	fn value_color(&self, _style: &Self::Style) -> Color {
		Color::new(0.93, 0.93, 0.93, 1.0)
	}
	fn selection_color(&self, _style: &Self::Style) -> Color {
		Color::new(0.63, 0.63, 0.93, 1.0)
	}
}
impl MyInputStyle {
	pub fn theme() -> iced::theme::TextInput {
		iced::theme::TextInput::Custom(Box::from(MyInputStyle))
	}
}
