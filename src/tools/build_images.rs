use lang_manager::LangManager;

/// Build all images for the referenced languages.
/// 
/// Use it with `cargo run build-images`.
/// 
/// This command should be used after pulling from the repository.
pub fn build_images() {
    let mut lang_manager = LangManager::new();
    lang_manager.check_available_languages();
}
