use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Create models directory if it doesn't exist
    let models_dir = Path::new("models");
    if !models_dir.exists() {
        fs::create_dir(models_dir).expect("Failed to create models directory");
    }
    
    let model_name = "en_GB-alba-medium";
    let model_path = models_dir.join(format!("{}.onnx", model_name));
    let config_path = models_dir.join(format!("{}.onnx.json", model_name));
    
    // Download model files if they don't exist
    if !model_path.exists() {
        println!("Downloading Alba model...");
        let model_url = format!("https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_GB/alba/medium/{}.onnx", model_name);
        download_file(&model_url, &model_path);
    }
    
    if !config_path.exists() {
        println!("Downloading Alba model config...");
        let config_url = format!("https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_GB/alba/medium/{}.onnx.json", model_name);
        download_file(&config_url, &config_path);
    }
    
    println!("cargo:rustc-env=MODEL_PATH={}", model_path.to_string_lossy());
}

fn download_file(url: &str, path: &Path) {
    let output = Command::new("curl")
        .arg("-L")
        .arg("-o")
        .arg(path)
        .arg(url)
        .output();
    
    match output {
        Ok(output) => {
            if output.status.success() {
                println!("Successfully downloaded {}", path.display());
            } else {
                eprintln!("Failed to download {}: {}", url, String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => {
            eprintln!("Failed to execute curl: {}", e);
            eprintln!("Please install curl or manually download the model files");
        }
    }
} 