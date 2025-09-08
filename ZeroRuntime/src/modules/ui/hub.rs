use std::fs;
use std::path::Path;

pub fn find_projects(paths: Vec<String>) -> Vec<String> {
    let mut valid_projects = Vec::new();
    for path in paths {
        if let Ok(entries) = fs::read_dir(&path) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_dir() {
                        if let Ok(sub_entries) = fs::read_dir(entry.path()) {
                            for sub_entry in sub_entries.flatten() {
                                if let Ok(sub_metadata) = sub_entry.metadata() {
                                    if sub_metadata.is_file()
                                        && sub_entry.path().extension().and_then(|e| e.to_str()) == Some("zero")
                                        && sub_entry.file_name() == "Project"
                                    {
                                        if let Some(path_str) = entry.path().to_str() {
                                            valid_projects.push(path_str.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    valid_projects
}

pub fn render_hub(ctx: &egui::Context) {
    let projects = find_projects(vec!["/home/kirby/ZeroG/Projects".to_string()]);
    
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Projects");
        ui.separator();
        
        for project in &projects {
            ui.horizontal(|ui| {
                let project_name = Path::new(project)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown Project");
                
                if ui.button(project_name).clicked() {
                    println!("Selected project: {}", project);
                }
                
                ui.label(project);
            });
        }
        
        if projects.is_empty() {
            ui.label("No projects found");
        }
    });
}
