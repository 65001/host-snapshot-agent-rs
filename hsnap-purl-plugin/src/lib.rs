use packageurl::PackageUrl;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "details")]
pub enum SoftwareComponent {
    Purl(PackageUrl<'static>),
    WindowsComponent {
        name: String,
        version: String,
        publisher: Option<String>,
    },
}

pub mod plugins;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum FileLocation {
    /// A full absolute path to the file (e.g., "/etc/hosts")
    AbsolutePath(String),
    /// A path relative to the agent's execution directory
    RelativePath(String),
    /// A binary name to look for in the system $PATH (e.g., "nginx")
    Path(String),
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum Probe {
    /// Check for a file existence
    File(FileLocation),
    /// Check for a Windows Registry Key existence (Windows only)
    WindowsRegistry(String),
    /// Execute a command and check for success
    Command(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct RegistryEntry {
    pub display_name: Option<String>,
    pub display_version: Option<String>,
    pub publisher: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum ProbeData {
    /// For file probes, provides the path to the found file.
    File(PathBuf),
    /// For command probes, provides the standard output.
    CommandOutput(String),
    /// For registry probes, provides the value/data found.
    RegistryEntries(Vec<RegistryEntry>),
}

/// Represents the result of a successful probe
pub struct ProbeResult {
    pub probe: Probe,
    pub data: ProbeData,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum Os {
    Linux,
    Windows,
    MacOS,
    Unknown,
}

pub trait Plugin {
    fn name(&self) -> &str;
    fn supported_os(&self) -> Option<Vec<Os>>;
    fn probes(&self) -> Vec<Probe>;
    fn extract(&self, found_probes: &[ProbeResult]) -> Vec<SoftwareComponent>;
}

fn get_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(plugins::WindowsRegistryPlugin),
        Box::new(plugins::RhelPlugin),
        Box::new(plugins::DebianPlugin),
    ]
}

pub fn run_plugins() -> Vec<SoftwareComponent> {
    // 1. Determine current OS
    let current_os = if cfg!(target_os = "windows") {
        Os::Windows
    } else if cfg!(target_os = "linux") {
        Os::Linux
    } else if cfg!(target_os = "macos") {
        Os::MacOS
    } else {
        Os::Unknown
    };

    let mut all_purls = Vec::new();
    let plugins = get_plugins();

    for plugin in plugins {
        // Filter by OS
        if let Some(supported) = plugin.supported_os() {
            if !supported.contains(&current_os) {
                continue;
            }
        }

        let mut probe_results = Vec::new();

        for probe in plugin.probes() {
            match &probe {
                Probe::File(loc) => {
                    let path_to_check = match loc {
                        FileLocation::AbsolutePath(p) => Some(PathBuf::from(p)),
                        FileLocation::RelativePath(p) => {
                            std::env::current_dir().ok().map(|cwd| cwd.join(p))
                        }
                        FileLocation::Path(bin_name) => {
                            if let Ok(paths) = std::env::var("PATH") {
                                std::env::split_paths(&paths).find_map(|p| {
                                    let full_path = p.join(bin_name);
                                    if full_path.exists() {
                                        Some(full_path)
                                    } else {
                                        None
                                    }
                                })
                            } else {
                                None
                            }
                        }
                    };

                    if let Some(path) = path_to_check {
                        if path.exists() {
                            probe_results.push(ProbeResult {
                                probe: probe.clone(),
                                data: ProbeData::File(path),
                            });
                        }
                    }
                }
                Probe::WindowsRegistry(key) => {
                    if cfg!(target_os = "windows") {
                        #[cfg(target_os = "windows")]
                        {
                            use winreg::enums::*;
                            use winreg::RegKey;

                            let (root_key, subkey_path) = if key.starts_with("HKLM\\") {
                                (
                                    RegKey::predef(HKEY_LOCAL_MACHINE),
                                    key.trim_start_matches("HKLM\\"),
                                )
                            } else if key.starts_with("HKCU\\") {
                                (
                                    RegKey::predef(HKEY_CURRENT_USER),
                                    key.trim_start_matches("HKCU\\"),
                                )
                            } else {
                                // Unknown root, skip or handle error? For now skip.
                                (RegKey::predef(HKEY_LOCAL_MACHINE), "")
                            };

                            if !subkey_path.is_empty() {
                                if let Ok(parent_key) = root_key.open_subkey(subkey_path) {
                                    let mut entries = Vec::new();
                                    for name in
                                        parent_key.enum_keys().map(|x| x.unwrap_or_default())
                                    {
                                        if let Ok(subkey) = parent_key.open_subkey(&name) {
                                            let display_name: Option<String> =
                                                subkey.get_value("DisplayName").ok();
                                            let display_version: Option<String> =
                                                subkey.get_value("DisplayVersion").ok();
                                            let publisher: Option<String> =
                                                subkey.get_value("Publisher").ok();

                                            if display_name.is_some() {
                                                entries.push(RegistryEntry {
                                                    display_name,
                                                    display_version,
                                                    publisher,
                                                });
                                            }
                                        }
                                    }

                                    if !entries.is_empty() {
                                        probe_results.push(ProbeResult {
                                            probe: probe.clone(),
                                            data: ProbeData::RegistryEntries(entries),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
                Probe::Command(cmd_str) => {
                    let output = if cfg!(target_os = "windows") {
                        Command::new("cmd").args(["/C", cmd_str]).output()
                    } else {
                        Command::new("sh").arg("-c").arg(cmd_str).output()
                    };

                    if let Ok(out) = output {
                        if out.status.success() {
                            probe_results.push(ProbeResult {
                                probe: probe.clone(),
                                data: ProbeData::CommandOutput(
                                    String::from_utf8_lossy(&out.stdout).to_string(),
                                ),
                            });
                        }
                    }
                }
            }
        }

        if !probe_results.is_empty() {
            let results = plugin.extract(&probe_results);
            all_purls.extend(results);
        }
    }
    all_purls
}
