use iced::{
    widget::{button, Button, Text, column, container, scrollable, row},
    Element, Length, Background, Border, Color
};

use crate::{
    ui::styles,
    editor::scene_components::SceneFile,
};

#[derive(Debug, Clone)]
pub enum Message {
    Play,
    Pause,
    ButtonPressed,
    SelectEntity(String),
}

/// Toolbar with play/pause buttons
pub fn toolbar() -> Element<'static, Message> {
    row![
        button(Text::new("▶ Play"))
            .on_press(Message::Play)
            .style(styles::primary_button()),
        button(Text::new("⏸ Pause"))
            .on_press(Message::Pause)
            .style(styles::primary_button()),
    ]
    .spacing(10)
    .padding(5)
    .into()
}

/// Hierarchy panel for displaying entities and cameras
pub fn hierarchy_panel(scene: &SceneFile) -> Element<'_, Message> {
    let mut items = column![];

    // Add section header for entities if there are any
    if !scene.entities.is_empty() {
        items = items.push(
            Text::new("Entities")
                .style(|theme| iced::widget::text::Style {
                    color: Some(Color::from_rgb(0.8, 0.8, 0.8)),
                })
        );
        
        // Add entity buttons
        for entity in &scene.entities {
            items = items.push(
                Button::new(Text::new(&entity.name))
                    .on_press(Message::SelectEntity(entity.name.clone()))
                    .style(styles::primary_button())
                    .width(Length::Fill)
            );
        }
    }

    // Add some spacing between sections if both exist
    if !scene.entities.is_empty() && !scene.cameras.is_empty() {
        items = items.push(
            container("")
                .height(Length::Fixed(10.0))
        );
    }

    // Add section header for cameras if there are any
    if !scene.cameras.is_empty() {
        items = items.push(
            Text::new("Cameras")
                .style(|theme| iced::widget::text::Style {
                    color: Some(Color::from_rgb(0.8, 0.8, 0.8)),
                })
        );
        
        // Add camera buttons
        for camera in &scene.cameras {
            items = items.push(
                Button::new(Text::new(&camera.name))
                    .on_press(Message::SelectEntity(camera.name.clone()))
                    .style(styles::primary_button())
                    .width(Length::Fill)
            );
        }
    }

    // Show a message if the scene is empty
    if scene.entities.is_empty() && scene.cameras.is_empty() {
        items = items.push(
            Text::new("No entities or cameras in scene")
                .style(|theme| iced::widget::text::Style {
                    color: Some(Color::from_rgb(0.6, 0.6, 0.6)),
                })
        );
    }

    scrollable(items.spacing(5))
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}
