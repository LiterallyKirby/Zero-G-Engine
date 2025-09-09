pub mod components;
pub mod layout;
pub mod styles;
pub mod theme;

#[derive(Debug, Clone)]
pub enum Message {
    Component(components::Message),
}
