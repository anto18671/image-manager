// Import necessary libraries
use eframe::egui;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// Define a struct to hold configuration data
#[derive(Serialize, Deserialize)]
struct Config {
    input_folder: PathBuf,
    destination_folders: Vec<PathBuf>,
    trash_folder: PathBuf,
}

// Define an enum to represent different states of the application
enum AppState {
    Configuration,
    ImageManagement,
}

// Main struct for the Image Manager application
struct ImageManager {
    config: Config,
    current_image: Option<egui::TextureHandle>,
    current_image_path: Option<PathBuf>,
    images: Vec<PathBuf>,
    current_index: usize,
    state: AppState,
    new_folder_path: String,
    undo_history: Vec<(PathBuf, PathBuf)>,
}

impl ImageManager {
    // Constructor for ImageManager
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config = Self::load_or_create_config();
        let images = Self::load_images_from_folder(&config.input_folder);

        Self {
            config,
            current_image: None,
            current_image_path: None,
            images,
            current_index: 0,
            state: AppState::Configuration,
            new_folder_path: String::new(),
            undo_history: Vec::new(),
        }
    }

    // Load existing config or create a new one
    fn load_or_create_config() -> Config {
        if let Ok(config_str) = fs::read_to_string("config.json") {
            if let Ok(config) = serde_json::from_str(&config_str) {
                return config;
            }
        }

        Config {
            input_folder: PathBuf::new(),
            destination_folders: Vec::new(),
            trash_folder: PathBuf::new(),
        }
    }

    // Save the current configuration to a file
    fn save_config(&self) {
        let config_str = serde_json::to_string_pretty(&self.config).unwrap();
        fs::write("config.json", config_str).expect("Failed to write config file");
    }

    // Load images from a specified folder
    fn load_images_from_folder(folder: &Path) -> Vec<PathBuf> {
        if !folder.exists() {
            return Vec::new();
        }
        fs::read_dir(folder)
            .unwrap_or_else(|_| panic!("Failed to read directory: {:?}", folder))
            .filter_map(|entry| {
                let path = entry.ok()?.path();
                if path.extension().map_or(false, |ext| {
                    matches!(
                        ext.to_str().unwrap().to_lowercase().as_str(),
                        "png" | "jpg" | "jpeg" | "webp" | "bmp" | "tiff" | "gif"
                    )
                }) {
                    Some(path)
                } else {
                    None
                }
            })
            .collect()
    }

    // Load the current image into memory
    fn load_current_image(&mut self, ctx: &egui::Context) {
        if let Some(path) = &self.current_image_path {
            match image::open(path) {
                Ok(image) => {
                    let size = [image.width() as _, image.height() as _];
                    let image_buffer = image.to_rgba8();
                    let pixels = image_buffer.into_raw();
                    self.current_image = Some(ctx.load_texture(
                        "current-image",
                        egui::ColorImage::from_rgba_unmultiplied(size, &pixels),
                        Default::default(),
                    ));
                }
                Err(e) => {
                    eprintln!("Failed to load image {:?}: {}", path, e);
                    self.current_image = None;
                }
            }
        }
    }

    // Move the current image to a specified folder
    fn move_to_folder(&mut self, folder: &Path) {
        if let Some(current_path) = &self.current_image_path {
            let new_path = folder.join(current_path.file_name().unwrap());
            fs::rename(current_path, &new_path).unwrap_or_else(|_| panic!("Failed to move file"));
            self.undo_history
                .push((new_path.clone(), current_path.clone())); // Store both paths
            self.images.remove(self.current_index);
            if self.current_index >= self.images.len() && !self.images.is_empty() {
                self.current_index = self.images.len() - 1;
            }
            self.current_image_path = None;
            self.current_image = None;
        }
    }

    // Delete (move to trash) the current image
    fn delete_current_image(&mut self) {
        if let Some(current_path) = &self.current_image_path {
            if !self.config.trash_folder.exists() {
                fs::create_dir(&self.config.trash_folder)
                    .unwrap_or_else(|_| panic!("Failed to create trash folder"));
            }
            let new_path = self
                .config
                .trash_folder
                .join(current_path.file_name().unwrap());
            fs::rename(current_path, &new_path)
                .unwrap_or_else(|_| panic!("Failed to move file to trash"));
            self.undo_history.push((new_path, current_path.clone()));
            self.images.remove(self.current_index);
            if self.current_index >= self.images.len() && !self.images.is_empty() {
                self.current_index = self.images.len() - 1;
            }
            self.current_image_path = None;
            self.current_image = None;
        }
    }

    // Undo the last action (move or delete)
    fn undo_action(&mut self) {
        if let Some((destination, source)) = self.undo_history.pop() {
            if destination.exists() {
                fs::rename(&destination, &source).unwrap_or_else(|_| panic!("Failed to undo move"));
                self.images.push(source.clone());
                self.current_index = self.images.len() - 1;
                self.current_image_path = Some(source);
                self.current_image = None;
            }
        }
    }

    // Display the configuration UI
    fn show_configuration_ui(&mut self, ui: &mut egui::Ui) {
        ui.add_space(20.0);
        egui::Frame::none()
            .inner_margin(epaint::Margin {
                left: 20.0,
                right: 20.0,
                top: 0.0,
                bottom: 0.0,
            })
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.set_min_width(300.0);
                    ui.heading("Configuration");
                    ui.add_space(10.0);

                    let mut input_folder = self.config.input_folder.clone();
                    let mut trash_folder = self.config.trash_folder.clone();

                    self.folder_selector(ui, "Input Folder:", &mut input_folder);
                    self.folder_selector(ui, "Trash Folder:", &mut trash_folder);

                    self.config.input_folder = input_folder;
                    self.config.trash_folder = trash_folder;

                    ui.add_space(20.0);
                    ui.heading("Destination Folders");
                    ui.add_space(10.0);

                    let mut folders_to_remove = Vec::new();
                    for (index, folder) in self.config.destination_folders.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(folder.to_string_lossy());
                            if ui.button("Remove").clicked() {
                                folders_to_remove.push(index);
                            }
                        });
                    }
                    for index in folders_to_remove.iter().rev() {
                        self.config.destination_folders.remove(*index);
                    }
                    if !folders_to_remove.is_empty() {
                        self.save_config();
                    }

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.label("New Folder:");
                        ui.text_edit_singleline(&mut self.new_folder_path);
                        if ui.button("Add").clicked() && !self.new_folder_path.is_empty() {
                            self.config
                                .destination_folders
                                .push(PathBuf::from(&self.new_folder_path));
                            self.new_folder_path.clear();
                            self.save_config();
                        }
                        if ui.button("Browse").clicked() {
                            if let Some(path) = FileDialog::new().pick_folder() {
                                self.config.destination_folders.push(path);
                                self.save_config();
                            }
                        }
                    });

                    ui.add_space(20.0);
                    if ui
                        .add_sized([250.0, 40.0], egui::Button::new("Start Image Management"))
                        .clicked()
                    {
                        self.images = Self::load_images_from_folder(&self.config.input_folder);
                        self.state = AppState::ImageManagement;
                    }
                });
            });
        ui.add_space(20.0);
    }

    // Display the image management UI
    fn show_image_management_ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui
                .add_sized([150.0, 40.0], egui::Button::new("Back to Config"))
                .clicked()
            {
                self.state = AppState::Configuration;
            }
            if ui
                .add_sized([150.0, 40.0], egui::Button::new("Undo"))
                .clicked()
            {
                self.undo_action();
                self.load_current_image(ctx);
            }
            if ui
                .add_sized([150.0, 40.0], egui::Button::new("Delete"))
                .clicked()
            {
                self.delete_current_image();
            }
        });

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if let Some(image) = &self.current_image {
                ui.image(image);
            }

            let destination_folders: Vec<_> = self.config.destination_folders.clone();
            ui.vertical(|ui| {
                for folder in destination_folders {
                    let button_text = folder.file_name().unwrap().to_str().unwrap();
                    let button = egui::Button::new(button_text).min_size(egui::vec2(100.0, 28.0));

                    if ui.add_sized([ui.available_width(), 18.0], button).clicked() {
                        self.move_to_folder(&folder);
                    }
                }
            });
        });

        ui.add_space(20.0);

        if ui
            .add_sized([150.0, 40.0], egui::Button::new("Undo"))
            .clicked()
        {
            self.undo_action();
            self.load_current_image(ctx);
        }
    }

    // Helper function to create a folder selector UI
    fn folder_selector(&mut self, ui: &mut egui::Ui, label: &str, path: &mut PathBuf) {
        ui.horizontal(|ui| {
            ui.label(label);
            if ui.button("Browse").clicked() {
                if let Some(new_path) = FileDialog::new().pick_folder() {
                    *path = new_path;
                    self.save_config();
                }
            }
            ui.label(path.to_string_lossy());
        });
    }
}

// Implement the eframe::App trait for ImageManager
impl eframe::App for ImageManager {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().override_text_style = Some(egui::TextStyle::Heading);
            ui.style_mut().spacing.item_spacing = egui::vec2(10.0, 10.0);

            match self.state {
                AppState::Configuration => self.show_configuration_ui(ui),
                AppState::ImageManagement => self.show_image_management_ui(ctx, ui),
            }
        });

        if matches!(self.state, AppState::ImageManagement)
            && self.current_image.is_none()
            && !self.images.is_empty()
        {
            self.current_image_path = Some(self.images[self.current_index].clone());
            self.load_current_image(ctx);
        }
    }
}

// Main function to run the application
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        ..Default::default()
    };
    eframe::run_native(
        "Image Manager",
        options,
        Box::new(|cc| Ok(Box::new(ImageManager::new(cc)))),
    )
}
