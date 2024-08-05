pub mod error;
pub mod win32;

#[cfg(target_os = "windows")]
pub use win32::implements::NativeWindow as WindowHandler;
