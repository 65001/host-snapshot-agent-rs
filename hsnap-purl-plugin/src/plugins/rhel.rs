use crate::{Os, Plugin, Probe, ProbeData, ProbeResult, SoftwareComponent};
use packageurl::PackageUrl;

pub struct RhelPlugin;

impl Plugin for RhelPlugin {
    fn name(&self) -> &str {
        "rhel-rpm"
    }

    fn supported_os(&self) -> Option<Vec<Os>> {
        Some(vec![Os::Linux])
    }

    fn probes(&self) -> Vec<Probe> {
        vec![Probe::Command("rpm -qa --qf '%{NAME}|%{VERSION}|%{RELEASE}|%{ARCH}\\n'".to_string())]
    }

    fn extract(&self, found_probes: &[ProbeResult]) -> Vec<SoftwareComponent> {
        let mut components = Vec::new();
        for result in found_probes {
            if let ProbeData::CommandOutput(output) = &result.data {
                for line in output.lines() {
                    let parts: Vec<&str> = line.split('|').collect();
                    if parts.len() >= 4 {
                        if let Ok(mut purl) = PackageUrl::new("rpm".to_string(), parts[0].to_string()) {
                            purl.with_version(format!("{}-{}", parts[1], parts[2]));
                            let _ = purl.add_qualifier("arch", parts[3].to_string());
                            components.push(SoftwareComponent::Purl(purl));
                        }
                    }
                }
            }
        }
        components
    }
}
