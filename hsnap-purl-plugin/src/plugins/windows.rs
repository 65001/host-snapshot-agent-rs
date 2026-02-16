use crate::{Os, Plugin, Probe, ProbeResult, SoftwareComponent};


pub struct WindowsRegistryPlugin;

impl Plugin for WindowsRegistryPlugin {
    fn name(&self) -> &str {
        "windows-registry"
    }

    fn supported_os(&self) -> Option<Vec<Os>> {
        Some(vec![Os::Windows])
    }

    fn probes(&self) -> Vec<Probe> {
        vec![
            Probe::WindowsRegistry("HKLM\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall".to_string()),
            Probe::WindowsRegistry("HKLM\\SOFTWARE\\Wow6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall".to_string()),
            Probe::WindowsRegistry("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall".to_string()),
        ]
    }

    fn extract(&self, found_probes: &[ProbeResult]) -> Vec<SoftwareComponent> {
        use crate::ProbeData;
        let mut components = Vec::new();
        for result in found_probes {
            if let ProbeData::RegistryEntries(entries) = &result.data {
                for entry in entries {
                    if let Some(name) = &entry.display_name {
                        components.push(SoftwareComponent::WindowsComponent {
                            name: name.clone(),
                            version: entry.display_version.clone().unwrap_or_default(),
                            publisher: entry.publisher.clone(), 
                        });
                    }
                }
            }
        }
        components
    }
}
