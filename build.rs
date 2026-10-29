use std::env;

fn main() {
    // Get the target OS
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    
    println!("cargo:rerun-if-changed=build.rs");
    
    // Platform-specific configurations
    match target_os.as_str() {
        "windows" => {
            configure_windows();
        }
        "linux" => {
            configure_linux();
        }
        "macos" => {
            configure_macos();
        }
        _ => {
            println!("cargo:warning=Unsupported target OS: {}", target_os);
        }
    }
    
    // Architecture-specific optimizations
    match target_arch.as_str() {
        "x86_64" => {
            println!("cargo:rustc-cfg=arch_x86_64");
        }
        "aarch64" => {
            println!("cargo:rustc-cfg=arch_aarch64");
        }
        _ => {
            println!("cargo:warning=Unsupported target architecture: {}", target_arch);
        }
    }
    
    // Enable console on debug builds for Windows
    #[cfg(debug_assertions)]
    if target_os == "windows" {
        println!("cargo:rustc-link-arg=/SUBSYSTEM:CONSOLE");
    }
    
    // Set optimization flags
    set_optimization_flags(&target_os, &target_arch);
    
    // Detect available CPU features
    detect_cpu_features();
}

fn configure_windows() {
    println!("cargo:rustc-cfg=target_windows");
    
    // Windows-specific optimizations
    println!("cargo:rustc-link-arg=/OPT:REF");
    println!("cargo:rustc-link-arg=/OPT:ICF");
    
    // Enable high DPI awareness
    println!("cargo:rustc-link-arg=/MANIFESTUAC:level='asInvoker'");
    
    // Link against Windows-specific libraries
    println!("cargo:rustc-link-lib=user32");
    println!("cargo:rustc-link-lib=shell32");
    println!("cargo:rustc-link-lib=ole32");
    println!("cargo:rustc-link-lib=comctl32");
    
    // Set Windows version compatibility
    println!("cargo:rustc-cfg=windows_7_compatible");
}

fn configure_linux() {
    println!("cargo:rustc-cfg=target_linux");
    
    // Check for required system libraries
    match pkg_config::Config::new().probe("gtk+-3.0") {
        Ok(_) => {
            println!("cargo:rustc-cfg=gtk3_available");
        }
        Err(_) => {
            println!("cargo:warning=GTK3 development libraries not found. GUI may not work properly.");
        }
    }
    
    // Link against Linux-specific libraries
    println!("cargo:rustc-link-lib=pthread");
    println!("cargo:rustc-link-lib=dl");
    println!("cargo:rustc-link-lib=m");
    
    // Enable Linux-specific features
    println!("cargo:rustc-cfg=unix_permissions");
    println!("cargo:rustc-cfg=linux_filesystems");
}

fn configure_macos() {
    println!("cargo:rustc-cfg=target_macos");
    
    // macOS-specific frameworks
    println!("cargo:rustc-link-lib=framework=Cocoa");
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-lib=framework=CoreServices");
    
    // Enable macOS-specific features
    println!("cargo:rustc-cfg=unix_permissions");
    println!("cargo:rustc-cfg=macos_bundle");
    
    // Set minimum macOS version
    println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=10.12");
}

fn set_optimization_flags(target_os: &str, target_arch: &str) {
    // Enable link-time optimization hints
    println!("cargo:rustc-cfg=lto_enabled");
    
    // Platform and architecture specific optimizations
    match (target_os, target_arch) {
        ("windows", "x86_64") => {
            println!("cargo:rustc-cfg=windows_x64_optimized");
        }
        ("linux", "x86_64") => {
            println!("cargo:rustc-cfg=linux_x64_optimized");
        }
        ("macos", "x86_64") => {
            println!("cargo:rustc-cfg=macos_x64_optimized");
        }
        ("macos", "aarch64") => {
            println!("cargo:rustc-cfg=macos_arm64_optimized");
        }
        _ => {}
    }
    
    // Debug vs Release configurations
    if env::var("PROFILE").unwrap() == "release" {
        println!("cargo:rustc-cfg=release_build");
        
        // Release-specific optimizations
        if target_os == "windows" {
            println!("cargo:rustc-link-arg=/LTCG");
        }
    } else {
        println!("cargo:rustc-cfg=debug_build");
    }
}

#[cfg(feature = "bundle")]
fn setup_app_bundle() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    
    if target_os == "macos" {
        // macOS app bundle configuration
        println!("cargo:rustc-env=CFBundleName=DiskUsage");
        println!("cargo:rustc-env=CFBundleDisplayName=DiskUsage");
        println!("cargo:rustc-env=CFBundleIdentifier=com.diskusage.app");
        println!("cargo:rustc-env=CFBundleVersion=1.0.0");
        println!("cargo:rustc-env=CFBundleShortVersionString=1.0.0");
    }
}

// Conditional compilation for build dependencies
#[cfg(feature = "embed_resources")]
fn embed_resources() {
    use std::path::Path;
    
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let resources_dir = Path::new(&manifest_dir).join("resources");
    
    if resources_dir.exists() {
        println!("cargo:rerun-if-changed=resources");
        
        // Windows resource embedding
        #[cfg(target_os = "windows")]
        {
            let rc_file = resources_dir.join("app.rc");
            if rc_file.exists() {
                println!("cargo:rerun-if-changed={}", rc_file.display());
            }
        }
        
        // Embed icons and other resources
        let icon_file = resources_dir.join("icon.ico");
        if icon_file.exists() {
            println!("cargo:rustc-env=APP_ICON={}", icon_file.display());
        }
    }
}

// Helper function to detect available CPU features
fn detect_cpu_features() {
    // Enable SIMD optimizations where available
    if env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default().contains("sse2") {
        println!("cargo:rustc-cfg=simd_sse2");
    }
    
    if env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default().contains("avx") {
        println!("cargo:rustc-cfg=simd_avx");
    }
    
    if env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default().contains("neon") {
        println!("cargo:rustc-cfg=simd_neon");
    }
}