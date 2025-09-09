use crate::Message; // <- top-level app Message
use crate::ui;
use crate::ui::{components, styles};
use iced::Element;
use iced::Length;
use iced::border;
use iced::widget::Text;
use iced::widget::row;
use iced::widget::text;
use iced::widget::{Button, column, container, scrollable}; // <- include scrollable
use iced::{Background, Border, Color};

use crate::editor::scene_components::SceneFile;

/// Main layout function
pub fn main_layout(scene: &crate::editor::scene_components::SceneFile) -> Element<'_, Message> {


                    println!("{:?}", scene);
    container(
        column![
            // Toolbar at the top
            container(components::toolbar().map(|m| Message::Ui(ui::Message::Component(m))))
                .width(Length::Fill)
                .padding(5),
            // Main row: hierarchy, main view, inspector
            row![
                // Hierarchy panel
                container(

                    components::hierarchy_panel(scene)
                        .map(|m| Message::Ui(ui::Message::Component(m)))
                )
                .width(Length::FillPortion(1))
                .padding(5)
                .style(|_theme| container::Style {
                    border: Border {
                        width: 2.0,
                        color: Color::from_rgb(0.5, 0.5, 0.5),
                        ..Default::default()
                    },
                    background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
                    ..Default::default()
                }),
                // Main view placeholder
                container(Text::new("Main view"))
                    .width(Length::FillPortion(3))
                    .padding(5)
                    .style(|_theme| container::Style {
                        border: Border {
                            width: 2.0,
                            color: Color::from_rgb(0.5, 0.5, 0.5),
                            ..Default::default()
                        },
                        background: Some(Background::Color(Color::from_rgb(0.15, 0.15, 0.15))),
                        ..Default::default()
                    }),
                // Inspector placeholder
                container(Text::new("Inspector"))
                    .width(Length::FillPortion(1))
                    .padding(5)
                    .style(|_theme| container::Style {
                        border: Border {
                            width: 2.0,
                            color: Color::from_rgb(0.5, 0.5, 0.5),
                            ..Default::default()
                        },
                        background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
                        ..Default::default()
                    }),
            ]
            .spacing(10),
        ]
        .spacing(10),
    )
    .padding(10)
    .into()
}
