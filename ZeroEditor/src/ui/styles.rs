use iced::widget::button;

pub fn primary_button<'a>() -> impl Fn(&iced::Theme, button::Status) -> button::Style + 'a {
    |_, status| {
        let mut style = button::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.2, 0.5, 0.8))),
            text_color: iced::Color::WHITE,
            ..Default::default()
        };

        match status {
            button::Status::Pressed => {
                style.background = Some(iced::Background::Color(iced::Color::from_rgb(0.1, 0.3, 0.6)));
            }
            button::Status::Hovered => {
                style.background = Some(iced::Background::Color(iced::Color::from_rgb(0.25, 0.55, 0.85)));
            }
            _ => {}
        }

        style
    }
}
