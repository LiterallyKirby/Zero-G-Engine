use anyhow::Result;
use editor::scene_components::*;
use iced::{
    Element, Settings, Theme, application, executor, theme,
    widget::{Button, button, column, text},
};
use std::{default, env, fs};
use ui::components::Message::*;

mod editor;
mod ui;

#[derive(Debug)]
struct App {
    scene: SceneFile,
}

#[derive(Debug, Clone)]
enum Message {
    Ui(ui::Message),
    GlobalTest,
}


impl Default for App {
    fn default() -> Self {
        let args: Vec<String> = std::env::args().collect();

        let scene = if args.len() > 1 {
            let scene_path = &args[1];
            println!("Loading scene from: {}", scene_path);

            match Self::load_scene_file_debug(scene_path) {
                Ok(scene) => {
                    println!("Successfully loaded scene file");
                    scene
                }
                Err(e) => {
                    eprintln!("Failed to load scene file '{}': {}", scene_path, e);
                    eprintln!("Starting with default empty scene");
                    SceneFile::default()
                }
            }
        } else {
            println!("No scene file specified, starting with default scene");
            println!("Usage: cargo run <scene_file.json>");
            SceneFile::default()
        };

        Self { scene }
    }
}

impl App {


    fn update(&mut self, message: Message) {
        match message {
            Message::Ui(ui::Message::Component(ButtonPressed)) => {
                println!("I like boys");
            }
            Message::Ui(ui::Message::Component(Play)) => {
                println!("I like to play");
            }
            Message::Ui(ui::Message::Component(Pause)) => {
                println!("Pause");
            }
            Message::GlobalTest => {
                println!("works");
            }
            Message::Ui(ui::Message::Component(SelectEntity(s))) => {
                println!("Selected: {}", s);
            }
        }
    }

    pub fn load_scene_file_debug(path: &str) -> Result<SceneFile, String> {
        println!("=== DEBUG: Starting to load scene file ===");
        
        // First, check if file exists
        if !std::path::Path::new(path).exists() {
            return Err(format!("File does not exist: {}", path));
        }
        
        let data = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
        
        // Debug: Print the raw JSON data
        println!("Raw JSON data ({} bytes):", data.len());
        println!("{}", data);
        println!("=== End of raw JSON ===");
        
        // Try to parse as generic JSON first to see the structure
        let json_value: serde_json::Value = serde_json::from_str(&data)
            .map_err(|e| format!("Invalid JSON syntax: {}", e))?;
        
        println!("Parsed JSON structure:");
        println!("{:#?}", json_value);
        println!("=== End of parsed JSON structure ===");
        
        // Now try to deserialize to your struct
        match serde_json::from_str::<SceneFile>(&data) {
            Ok(scene) => {
                println!("Successfully deserialized SceneFile:");
                println!("- Entities count: {}", scene.entities.len());
                println!("- Cameras count: {}", scene.cameras.len());
                println!("Full SceneFile: {:#?}", scene);
                Ok(scene)
            }
            Err(e) => {
                println!("Failed to deserialize to SceneFile:");
                println!("Error: {}", e);
                println!("Error details: {:#?}", e);
                Err(format!("Deserialization failed: {}", e))
            }
        }
    }

    pub fn load_scene_file(path: &str) -> Result<SceneFile, String> {
        let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
        serde_json::from_str(&data).map_err(|e| e.to_string())
    }

    pub fn parse_scene_file(&mut self, path: &str) -> Result<(), String> {
        self.scene = Self::load_scene_file(path)?;
        Ok(())
    }
    
    fn view(&self) -> Element<Message> {
        ui::layout::main_layout(&self.scene)
    }
}

fn main() -> iced::Result {
    application("Clean Iced App", App::update, App::view)
        .theme(|_| ui::theme::app_theme()) // global theme
        .run()
}
