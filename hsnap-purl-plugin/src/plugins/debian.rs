use crate::{Os, Plugin, Probe, ProbeData, ProbeResult, SoftwareComponent};
use packageurl::PackageUrl;

pub struct DebianPlugin;

impl Plugin for DebianPlugin {
    fn name(&self) -> &str {
        "debian-dpkg"
    }

    fn supported_os(&self) -> Option<Vec<Os>> {
        Some(vec![Os::Linux])
    }

    fn probes(&self) -> Vec<Probe> {
        vec![Probe::Command("dpkg-query -W -f='${Package}|${Version}|${Architecture}\\n'".to_string())]
    }

    fn extract(&self, found_probes: &[ProbeResult]) -> Vec<SoftwareComponent> {
        let mut components = Vec::new();
        for result in found_probes {
            if let ProbeData::CommandOutput(output) = &result.data {
                for line in output.lines() {
                    let parts: Vec<&str> = line.split('|').collect();
                    if parts.len() >= 3 {
                        // Manually construct owned PackageUrl to ensure 'static lifetime
                        // We avoid PackageUrl::new because it might infer specific lifetime from &str args
                        // and we need to verify if it supports ownership transfer easily.
                        // Struct instantiation is safer if fields are public.

                         if let Ok(mut purl) = PackageUrl::new("deb".to_string(), parts[0].to_string()) {
                            purl.with_version(parts[1].to_string());
                            let _ = purl.add_qualifier("arch", parts[2].to_string());
                            components.push(SoftwareComponent::Purl(purl));
                        }
                    }
                }
            }
        }
        components
    }
}
