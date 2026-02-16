use chrono::{DateTime, Utc};
use clap::Parser;
use serde::Serialize;
use sysinfo::{Components, Disks, Networks, System, Users};
use hsnap_purl_plugin::{self, SoftwareComponent};



#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "false")]
    pretty: bool,

    /// ID to map hsnap to a host. Defaults to hostname if not provided.
    #[arg(long)]
    id: Option<String>,

    /// URL to POST the JSON data to.
    #[arg(long)]
    url: Option<String>,
}

#[derive(Serialize)]
struct HostSnapshot {
    metadata: Metadata,
    hardware: HardwareInfo,
    operating_system: OperatingSystemInfo,
    network: NetworkInfo,
    storage: StorageInfo,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    services: Vec<String>, // Placeholder
    users: Vec<UserInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    software_components: Vec<SoftwareComponent>,
}


#[derive(Serialize)]
struct Metadata {
    // The user will provide this id, to map hsnap to a host. If not provided, the hsnap will use the hostname
    id: String,
    timestamp: DateTime<Utc>,
}

#[derive(Serialize)]
struct HardwareInfo {
    cpu_info: Vec<CpuInfo>,
    memory: MemoryInfo,
    components: Vec<ComponentInfo>,
}

#[derive(Serialize)]
struct CpuInfo {
    name: String,
    vendor_id: String,
    brand: String,
    frequency: u64,
    usage: f32,
}

#[derive(Serialize)]
struct MemoryInfo {
    total_memory: u64,
    used_memory: u64,
    total_swap: u64,
    used_swap: u64,
}

#[derive(Serialize)]
struct ComponentInfo {
    label: String,
    temperature: Option<f32>,
}

#[derive(Serialize)]
struct OperatingSystemInfo {
    os_name: Option<String>,
    os_version: Option<String>,
    kernel_version: Option<String>,
    host_name: Option<String>,
}

#[derive(Serialize)]
struct NetworkInfo {
    interfaces: Vec<NetworkInterface>,
}

#[derive(Serialize)]
struct NetworkInterface {
    name: String,
    mac_address: String,
    ips: Vec<String>,
}

#[derive(Serialize)]
struct StorageInfo {
    disks: Vec<DiskInfo>,
}

#[derive(Serialize)]
struct DiskInfo {
    name: String,
    kind: String,
    file_system: String,
    mount_point: String,
    total_space: u64,
    available_space: u64,
    is_removable: bool,
}

#[derive(Serialize)]
struct UserInfo {
    name: String,
    id: String,
    groups: Vec<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Initialize sysinfo structures
    let mut sys = System::new_all();
    sys.refresh_all();

    let disks = Disks::new_with_refreshed_list();
    let networks = Networks::new_with_refreshed_list();
    let components = Components::new_with_refreshed_list();
    let users = Users::new_with_refreshed_list();

    // Determine Host ID: Argument > Hostname > "unknown"
    let host_id = args
        .id
        .or_else(|| System::host_name())
        .unwrap_or_else(|| "unknown".to_string());

    // Collect Data
    let snapshot = HostSnapshot {
        metadata: Metadata {
            id: host_id,
            timestamp: Utc::now(),
        },
        hardware: HardwareInfo {
            cpu_info: sys
                .cpus()
                .iter()
                .map(|cpu| CpuInfo {
                    name: cpu.name().to_string(),
                    vendor_id: cpu.vendor_id().to_string(),
                    brand: cpu.brand().to_string(),
                    frequency: cpu.frequency(),
                    usage: cpu.cpu_usage(),
                })
                .collect(),
            memory: MemoryInfo {
                total_memory: sys.total_memory(),
                used_memory: sys.used_memory(),
                total_swap: sys.total_swap(),
                used_swap: sys.used_swap(),
            },
            components: components
                .iter()
                .map(|c| ComponentInfo {
                    label: c.label().to_string(),
                    temperature: c.temperature(),
                })
                .collect(),
        },
        operating_system: OperatingSystemInfo {
            os_name: System::name(),
            os_version: System::os_version(),
            kernel_version: System::kernel_version(),
            host_name: System::host_name(),
        },
        network: NetworkInfo {
            interfaces: networks
                .iter()
                .map(|(interface_name, network)| NetworkInterface {
                    name: interface_name.clone(),
                    mac_address: network.mac_address().to_string(),
                    ips: network
                        .ip_networks()
                        .iter()
                        .map(|ip| ip.addr.to_string())
                        .collect(),
                })
                .collect(),
        },
        storage: StorageInfo {
            disks: disks
                .iter()
                .map(|disk| DiskInfo {
                    name: disk.name().to_string_lossy().to_string(),
                    kind: format!("{:?}", disk.kind()),
                    file_system: disk.file_system().to_string_lossy().to_string(),
                    mount_point: disk.mount_point().to_string_lossy().to_string(),
                    total_space: disk.total_space(),
                    available_space: disk.available_space(),
                    is_removable: disk.is_removable(),
                })
                .collect(),
        },
        services: vec![], // Placeholder
        users: users
            .iter()
            .map(|user| UserInfo {
                name: user.name().to_string(),
                id: user.id().to_string(),
                groups: user.groups().iter().map(|g| g.name().to_string()).collect(),
            })
            .collect(),
        software_components: hsnap_purl_plugin::run_plugins(),
    };

    if let Some(url) = args.url {
        let client = reqwest::Client::new();
        match client.post(&url).json(&snapshot).send().await {
            Ok(res) => {
                if res.status().is_success() {
                    println!("Successfully sent snapshot to {}", url);
                } else {
                    eprintln!(
                        "Failed to send snapshot to {}: Status {}",
                        url,
                        res.status()
                    );
                }
            }
            Err(e) => eprintln!("Error sending snapshot to {}: {}", url, e),
        }
    } else {
        let output = if args.pretty {
            serde_json::to_string_pretty(&snapshot).unwrap()
        } else {
            serde_json::to_string(&snapshot).unwrap()
        };
        println!("{}", output);
    }
}

