# Image Manager

Image Manager is a Rust-based desktop application designed to streamline the process of organizing and labeling images for image classification datasets. This tool is particularly useful for researchers and developers working on machine learning projects that require large, well-organized image datasets.

## Features

- **User-friendly GUI**: Built with egui, providing a smooth and responsive user experience.
- **Flexible Configuration**: Easily set up input, output, and trash folders to suit your workflow.
- **Efficient Image Management**: Quickly view and categorize images with keyboard shortcuts.
- **Multiple Destination Folders**: Organize images into various categories or classes.
- **Undo Functionality**: Easily correct mistakes with the undo feature.
- **Trash Folder**: Safely remove unwanted images without permanent deletion.
- **Cross-platform**: Works on Windows, macOS, and Linux.

## Prerequisites

- Rust programming language (latest stable version)
- Cargo (Rust's package manager)

## Installation

1. Clone the repository:

   ```
   git clone https://github.com/anto18671/ImageManager
   cd ImageManager
   ```

2. Build the project:

   ```
   cargo build --release
   ```

3. Run the application:
   ```
   cargo run --release
   ```

## Usage

1. **Configuration**:

   - Set the input folder containing your images.
   - Set the trash folder for unwanted images.
   - Add destination folders for different image categories.

2. **Image Management**:

   - Navigate through images.
   - Move images to category folders.
   - Delete unwanted images (moves them to the trash folder).
   - Use the undo feature if you make a mistake.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [egui](https://github.com/emilk/egui) - The GUI framework used in this project.
- [image](https://github.com/image-rs/image) - Rust library for reading and writing images.
- [serde](https://github.com/serde-rs/serde) - Serialization framework for Rust.

## Support

If you encounter any problems or have any suggestions, please open an issue on the GitHub repository.
