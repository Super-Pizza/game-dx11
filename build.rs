use std::{env, ffi::OsStr, fs::read_dir, path::PathBuf, process::Command};
fn main() {
    let root_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut shader_dir = PathBuf::from(root_dir);
    shader_dir.push("src\\shaders");
    read_dir(shader_dir)
        .unwrap()
        .into_iter()
        .for_each(|shader| {
            let shader = shader.unwrap();
            if shader.path().extension().unwrap_or_else(|| OsStr::new("")) == OsStr::new("hlsl") {
                let shader_file = shader.file_name().into_string().unwrap();
                let kind = if shader_file.to_lowercase().contains("vertex") {
                    "vs_5_0"
                } else if shader_file.to_lowercase().contains("hull") {
                    "hs_5_0"
                } else if shader_file.to_lowercase().contains("domain") {
                    "ds_5_0"
                } else if shader_file.to_lowercase().contains("geometry") {
                    "gs_5_0"
                } else if shader_file.to_lowercase().contains("pixel") {
                    "ps_5_0"
                } else if shader_file.to_lowercase().contains("compute") {
                    "cs_5_0"
                } else {
                    panic!("Unknown shader type, please name it properly.")
                };
                let cmd = Command::new(
                    "C:\\Program Files (x86)\\Windows Kits\\10\\bin\\10.0.19041.0\\x64\\fxc.exe",
                )
                .args(&[
                    "/O2",
                    "/E",
                    "ShaderMain",
                    "/Fo",
                    &format!(
                        "{}\\{}.cso",
                        env::var("OUT_DIR").unwrap(),
                        shader
                            .file_name()
                            .into_string()
                            .unwrap()
                            .strip_suffix(".hlsl")
                            .unwrap()
                    ),
                    "/T",
                    kind,
                    "/nologo",
                    &format!(
                        "{}\\src\\shaders\\{}",
                        env::var("CARGO_MANIFEST_DIR").unwrap(),
                        shader.file_name().into_string().unwrap()
                    ),
                ])
                .output()
                .unwrap();
                if !cmd.status.success() {
                    panic!(
                        "Command failed with: O:{}, E:{}",
                        String::from_utf8(cmd.stdout).unwrap(),
                        String::from_utf8(cmd.stderr).unwrap()
                    )
                }
            }
        });
}
