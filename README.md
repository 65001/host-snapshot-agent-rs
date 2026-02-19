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
* Windows

## Covered Architectures
* x86_64
* arm64

## Capabilities
* Extract metadata from the host
* Extract hardware information from the host
* Extract software information from the host
* Extract network information from the host
* Extract storage information from the host
* Extract user information from the host
* Extract operating system information from the host

## Security

The agent will only run when invoked by a user or a scheduler, and will immediately terminate. It is not written to be persistent.

## Connections

The agent will only make a connection to the specified url, only when the `--url` flag is passed. Otherwise it will only write to `stdout`.


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
