#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[cfg(not(debug_assertions))]
fn hide_console_window() {
    use winapi::um::wincon::GetConsoleWindow;
    use winapi::um::winuser::{ShowWindow, SW_HIDE};

    unsafe {
        let console_window = GetConsoleWindow();
        ShowWindow(console_window, SW_HIDE);
    }
}

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::path::Path;
use std::env;

use std::process::Command;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    #[cfg(not(debug_assertions))]
    hide_console_window();
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(420.0, 240.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Espanso Helper Egui",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::<MyApp>::default()
        }),
    )
}

struct MyApp {
    trigger: String,
    replace: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            trigger: "".to_owned(),
            replace: "".to_owned(),
        }
    }
}


impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hier Trigger und Replace Text eintragen.");

            ui.columns(2, |columns| {
                // First column
                columns[0].add(
                    egui::TextEdit::singleline(&mut self.trigger)
                        .hint_text("Trigger")
                        .desired_width(200.0),
                );
                columns[0].add(
                    egui::TextEdit::multiline(&mut self.replace)
                        .hint_text("Replace")
                        .desired_width(200.0)
                        .desired_rows(5),
                );

                // Second column
                let button_response = columns[1].add(egui::Button::new("Append"));
                if button_response.clicked() {
                    if let Err(err) = self.append_to_espanso_custom() {
                        eprintln!("Failed to append to espanso custom: {}", err);
                    }
                }
                let button_response = columns[1].add(egui::Button::new("Open Folder"));
                if button_response.clicked() {
                    if let Err(err) = open_espanso_custom_folder() {
                        eprintln!("Failed to open folder: {}", err);
                    } else {
                        println!("Folder open");
                    }
                }
            });

            //ui.label(format!("Hello '{}'", self.trigger));
        });
    }
}




impl MyApp {
    fn append_to_espanso_custom(&self) -> Result<(), std::io::Error> {
        if self.trigger.is_empty() {
            return Ok(());
        }
        
        let espanso_custom_path = get_espanso_custom_path()?;
        println!("espanso_custom_path: {}", espanso_custom_path.display());
        
        let file_exists = Path::new(&espanso_custom_path).exists();
        
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(espanso_custom_path.clone())?;
        
        if !file_exists {
            file.write_all(b"matches:\n")?;
            println!("Created new espanso_custom file.");
        } else {
            println!("espanso_custom file already exists.");
        }
        
        let formatted_replace = if self.replace.contains('\n') {
            let indented_replace = self
                .replace
                .lines()
                .map(|line| format!("    {}", line))
                .collect::<Vec<_>>()
                .join("\n");
            format!("\n- trigger: '{}'\n  replace: |\n{}\n", self.trigger, indented_replace)
        } else {
            format!("\n- trigger: '{}'\n  replace: '{}'\n", self.trigger, self.replace)
        };
        
        file.write_all(formatted_replace.as_bytes())?;
        
        Ok(())
    }  
}
fn get_espanso_custom_path() -> Result<PathBuf, std::io::Error> {
    let home_dir = dirs::home_dir().ok_or(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Failed to get home directory",
    ))?;
    let mut config_dir = home_dir.join(env::var("APPDATA").expect("Failed to get APPDATA environment variable"));
    config_dir.push("espanso");
    config_dir.push("match");
    fs::create_dir_all(&config_dir)?;
    let custom_path = config_dir.join("custom.yml");
    println!("espanso custom path: {}", custom_path.display());
    Ok(custom_path)
}
fn open_espanso_custom_folder() -> Result<(), std::io::Error> {
    let espanso_custom_path = get_espanso_custom_path()?;
    let parent_folder = espanso_custom_path.parent().ok_or(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Failed to get parent folder",
    ))?;
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("explorer").arg(parent_folder).output()?;
        if output.status.success() {
            println!("Folder open");
        } else {
            println!("Failed to open folder");
        }
    }
    #[cfg(target_os = "linux")]
    {
        let output = Command::new("xdg-open").arg(parent_folder).output()?;
        if output.status.success() {
            println!("Folder open");
        } else {
            println!("Failed to open folder");
        }
    }
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("open").arg(parent_folder).output()?;
        if output.status.success() {
            println!("Folder open");
        } else {
            println!("Failed to open folder");
        }
    }
    Ok(())
}