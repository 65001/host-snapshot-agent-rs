pub mod windows;
pub mod rhel;
pub mod debian;

pub use windows::WindowsRegistryPlugin;
pub use rhel::RhelPlugin;
pub use debian::DebianPlugin;
