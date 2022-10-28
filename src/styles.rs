use iced::{
	Background,
	Color,
	button::Style as ButtonStyle,
	button::StyleSheet as ButtonStyleSheet,
	text_input::Style as TextInputStyle,
	text_input::StyleSheet as TextInputStyleSheet
};


pub struct MyBtnStyle;
impl ButtonStyleSheet for MyBtnStyle {
    fn active(&self) -> ButtonStyle {
        let mut s = ButtonStyle::default();
        s.border_radius = 3.0;
        s.background = Some(Background::Color(Color::new(0.44, 0.34, 0.64, 1.0)));
        s.border_color = Color::new(0.73, 0.73, 0.73, 1.0);
        s.text_color = Color::new(0.93, 0.93, 0.93, 1.0);
        s
    }

    fn hovered(&self) -> ButtonStyle {
        let mut s = self.active();
        s.background = Some(Background::Color(Color::new(0.54, 0.44, 0.74, 1.0)));
        s
    }

    fn pressed(&self) -> ButtonStyle {
        self.active()
    }
}

pub struct MyOtherBtnStyle;
impl ButtonStyleSheet for MyOtherBtnStyle {
    fn active(&self) -> ButtonStyle {
        let mut s = ButtonStyle::default();
        s.border_radius = 3.0;
        s.background = Some(Background::Color(Color::new(0.34, 0.34, 0.34, 1.0)));
        s.border_color = Color::new(0.73, 0.73, 0.73, 1.0);
        s.text_color = Color::new(0.93, 0.93, 0.93, 1.0);
        s
    }

    fn hovered(&self) -> ButtonStyle {
        let mut s = self.active();
        s.background = Some(Background::Color(Color::new(0.54, 0.54, 0.54, 1.0)));
        s
    }

    fn pressed(&self) -> ButtonStyle {
        self.active()
    }
}


pub struct MyInputStyle;
impl TextInputStyleSheet for MyInputStyle {
	fn active(&self) -> TextInputStyle {
		let mut s = TextInputStyle::default();
		s.background = Background::Color(Color::new(0.2, 0.2, 0.2, 1.0));
		s.border_radius = 0.0;
		s
	}
	fn focused(&self) -> TextInputStyle {
		self.active()
	}
	fn placeholder_color(&self) -> Color {
		Color::new(0.63, 0.63, 0.63, 1.0)
	}

	fn value_color(&self) -> Color {
		Color::new(0.93, 0.93, 0.93, 1.0)
	}
	fn selection_color(&self) -> Color {
		Color::new(0.63, 0.63, 0.93, 1.0)
	}
}