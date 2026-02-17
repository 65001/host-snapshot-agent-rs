use chrono::{DateTime, Utc};
use clap::Parser;
use hsnap_purl_plugin::{self, SoftwareComponent};
use reqwest::Client;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::{Pkcs1v15Sign, RsaPrivateKey};
use serde::{Deserialize, Serialize};
use sysinfo::{Components, Disks, Networks, System, Users};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// ID to map hsnap to a host. Defaults to hostname if not provided.
    #[arg(long)]
    id: Option<String>,

    /// URL to POST the JSON data to.
    #[arg(long)]
    url: Option<String>,

    /// The private key used to sign this data, as a string.
    #[arg(long)]
    signing_key: Option<String>,
}

#[derive(Serialize)]
struct SignedSnapshot {
    snapshot: HostSnapshot,
    // The signature is serialized as a Hex string (default for rsa+serde)
    signature: String,
}

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
struct Metadata {
    // The user will provide this id, to map hsnap to a host. If not provided, the hsnap will use the hostname
    id: String,
    timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone)]
struct HardwareInfo {
    cpu_info: Vec<CpuInfo>,
    memory: MemoryInfo,
    components: Vec<ComponentInfo>,
}

#[derive(Serialize, Deserialize, Clone)]
struct CpuInfo {
    name: String,
    vendor_id: String,
    brand: String,
    frequency: u64,
    usage: f32,
}

#[derive(Serialize, Deserialize, Clone)]
struct MemoryInfo {
    total_memory: u64,
    used_memory: u64,
    total_swap: u64,
    used_swap: u64,
}

#[derive(Serialize, Deserialize, Clone)]
struct ComponentInfo {
    label: String,
    temperature: Option<f32>,
}

#[derive(Serialize, Deserialize, Clone)]
struct OperatingSystemInfo {
    os_name: Option<String>,
    os_version: Option<String>,
    kernel_version: Option<String>,
    host_name: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct NetworkInfo {
    interfaces: Vec<NetworkInterface>,
}

#[derive(Serialize, Deserialize, Clone)]
struct NetworkInterface {
    name: String,
    mac_address: String,
    ips: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct StorageInfo {
    disks: Vec<DiskInfo>,
}

#[derive(Serialize, Deserialize, Clone)]
struct DiskInfo {
    name: String,
    kind: String,
    file_system: String,
    mount_point: String,
    total_space: u64,
    available_space: u64,
    is_removable: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct UserInfo {
    name: String,
    id: String,
    groups: Vec<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Normal Capture Mode (with optional signing)
    let snapshot: HostSnapshot = capture_snapshot(&args).await;

    let signed_snapshot = match &args.signing_key {
        Some(private_key_pem) => {
            let private_key = RsaPrivateKey::from_pkcs1_pem(&private_key_pem)
                .expect("Failed to parse private key");
            let schema = Pkcs1v15Sign::new_unprefixed();
            let snapshot_bytes =
                serde_json::to_vec(&snapshot).expect("Failed to serialize snapshot");
            let signature = private_key
                .sign(schema, snapshot_bytes.as_slice())
                .expect("Unable to sign snapshot with private key");

            let string_signature = hex::encode(&signature);

            Some(SignedSnapshot {
                snapshot: snapshot.clone(),
                signature: string_signature,
            })
        }
        None => None,
    };

    match (&args.url, &signed_snapshot) {
        (Some(url), Some(signed_snapshot)) => {
            //Post the signed snapshot to the given url
            let client = reqwest::Client::new();
            post_data(client, url, signed_snapshot).await;
        }
        (Some(url), None) => {
            //Post the original snapshot to the given url
            let client = reqwest::Client::new();
            post_data(client, url, snapshot).await;
        }
        (None, Some(signed_snapshot)) => {
            //Pretty print the signed snapshot to stdout
            println!(
                "{}",
                serde_json::to_string_pretty(&signed_snapshot)
                    .expect("Failed to serialize signed snapshot")
            );
        }
        (None, None) => {
            //Pretty print the original snapshot to stdout
            println!(
                "{}",
                serde_json::to_string_pretty(&snapshot).expect("Failed to seralize snapshot")
            );
        }
    }
}

async fn post_data<T: Serialize + Sized>(client: Client, url: &String, json: T) {
    match client.post(url).json(&json).send().await {
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
        Err(e) => {
            eprintln!("Error sending snapshot to {}: {}", url, e)
        }
    }
}

async fn capture_snapshot(args: &Args) -> HostSnapshot {
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
        .clone()
        .or_else(|| System::host_name())
        .unwrap_or_else(|| "unknown".to_string());

    HostSnapshot {
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
    }
}
