fn main() {
    tauri_build::build();

    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = std::path::Path::new(&out_dir);

    let resources_src = manifest_dir.join("resources");
    let target_dir = out_path.join("../../../").join("Data");

    if resources_src.exists() {
        copy_sound_files(&resources_src, &target_dir).unwrap_or_default();
    }
}

fn copy_sound_files(
    src_dir: &std::path::Path,
    target_dir: &std::path::Path,
) -> std::io::Result<()> {
    let sound_src = src_dir.join("sound");
    let sound_target = target_dir.join("sound");

    if !sound_src.exists() {
        return Ok(());
    }

    std::fs::create_dir_all(&sound_target)?;

    for entry in std::fs::read_dir(&sound_src)? {
        let entry = entry?;
        let src = entry.path();
        if src.is_file() {
            let dest = sound_target.join(src.file_name().unwrap());
            std::fs::copy(&src, &dest)?;
        }
    }

    Ok(())
}
