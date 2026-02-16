# host-snapshot-agent-rs

## Objectives
* Provide metadata to a configurable server. This server will be configurable at build time and at runtime.
* The point of the agent is to provide a once daily snapshot of a configuration of a host
* Lightweight and non-intrusive
* Statically linked

## Covered Operating Systems
* Linux
    * RHEL
    * Ubuntu
    * Alpine
    * Debian
    * Amazon
* Windows

## Covered Architectures
* x86_64
* arm64

## Agent Todo Features
* Be able to extract metadata from the host
* Be able to extract hardware information from the host
* Be able to extract software information from the host
* Be able to extract network information from the host
* Be able to extract storage information from the host
* Be able to extract process information from the host
* Be able to extract service information from the host
* Be able to extract user information from the host
* Be able to extract system information from the host


## Architecture

### Plugin System
The agent uses a modular plugin system to extract software components (Package URLs) from the host.

#### Core Concept
- **Plugin Trait**: Defined in `hsnap-purl-plugin`. Each plugin implements:
    - `name()`: Unique identifier.
    - `supported_os()`: List of supported operating systems (or `None` for all).
    - `probes()`: List of checks (Files, Registry Keys, Commands) to run.
    - `extract()`: detailed logic to parse probe results into Package URLs (PURLs).

#### Included Plugins
- **RhelPlugin**: Detects RPM packages on Linux via `rpm -qa`.
- **DebianPlugin**: Detects Debian packages on Linux via `dpkg-query`.
- **WindowsRegistryPlugin**: Detects software on Windows via Registry.

### Package URL (PURL)
Software components are identified using the [Package URL](https://github.com/package-url/purl-spec) standard.
Example: `pkg:rpm/fedora/curl@7.50.3-1.fc25?arch=i386&distro=fedora-25`