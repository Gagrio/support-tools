# 🍑 BUTTCRACK - Batch Utility To Transfer Cluster Resources And Configs Kit

> **Don't let your cluster data slip through the cracks!** 🕵️

A blazingly fast 🦀 Rust-powered tool that collects arbitrary host paths and archives them into a compressed bundle for upload and investigation.

## ✨ Features

📁 **Path Flexible** - Collect any directory or file path from the host, not just predefined ones
🗜️ **Compressed Archives** - Creates `.tar.gz` archives with timestamped names for easy tracking
🔒 **Read-Only by Design** - Input paths are always mounted read-only in container mode
🐳 **Container Ready** - Runs as a privileged container with explicit host path mounts and SELinux support
⚙️ **Env Var Support** - Every flag has a `BC_*` env var equivalent for Ansible-driven automation
🔌 **stack-validation Native** - Output naming and archive format designed to plug directly into existing Ansible upload pipelines
🦀 **Fast & Safe** - Built with Rust for reliable, predictable behaviour under privileged execution

## 🚀 Quick Start

### Prerequisites

- 🐳 **Podman** installed on the host
- 📁 **Read access** to the paths you want to collect
- 📤 **Write access** to the output directory

### Quick Start with Container

```bash
# Collect a single path
podman run --rm --privileged \
           -v /var/lib/rancher/rke2/server/db/etcd:/var/lib/rancher/rke2/server/db/etcd:ro \
           -v /tmp:/tmp \
           registry.opensuse.org/isv/suse/edge/support-tools/images/buttcrack:latest \
           --paths /var/lib/rancher/rke2/server/db/etcd \
           --output /tmp

# Collect multiple paths
podman run --rm --privileged \
           -v /var/lib/rancher/rke2/server/db/etcd:/var/lib/rancher/rke2/server/db/etcd:ro \
           -v /var/log/pods:/var/log/pods:ro \
           -v /tmp:/tmp \
           registry.opensuse.org/isv/suse/edge/support-tools/images/buttcrack:latest \
           --paths /var/lib/rancher/rke2/server/db/etcd,/var/log/pods \
           --output /tmp
```

**Note:** Each input path must be explicitly bind-mounted into the container with `:ro`. The output directory must be mounted with write access. The `:Z` flag may be required on SELinux systems — see the [SELinux note](#-troubleshooting) below.

### Building Custom Container Image

```bash
# Build the container image
podman build -t buttcrack:custom .

# Run your custom image
podman run --rm --privileged \
           -v /var/lib/rancher/rke2/server/db/etcd:/var/lib/rancher/rke2/server/db/etcd:ro \
           -v /tmp:/tmp \
           buttcrack:custom \
           --paths /var/lib/rancher/rke2/server/db/etcd \
           --output /tmp
```

### Installation from Source

**Prerequisites:**
- 🦀 **Rust 1.70+** (install via [rustup](https://rustup.rs/))

```bash
# Clone the repository
git clone https://github.com/suse-edge/support-tools.git
cd support-tools/buttcrack

# Build the tool
cargo build --release

# Run it
cargo run -- --paths /var/lib/rancher/rke2/server/db/etcd --output /tmp
```

## 📖 Usage

### Basic Usage

```bash
# Collect a single path
buttcrack --paths /var/lib/rancher/rke2/server/db/etcd --output /tmp

# Collect multiple paths (comma-separated)
buttcrack --paths /var/lib/rancher/rke2/server/db/etcd,/var/log/pods --output /tmp

# Using environment variables
BC_PATHS=/var/lib/rancher/rke2/server/db/etcd BC_OUTPUT=/tmp buttcrack
```

### Command Line Options

| Option | Short | Env var | Description | Default |
|--------|-------|---------|-------------|---------|
| `--paths` | `-p` | `BC_PATHS` | **Required** Comma-separated list of host paths to collect | - |
| `--output` | `-o` | `BC_OUTPUT` | Output directory for the archive | `/tmp` |
| `--verbose` | `-v` | `BC_VERBOSE` | Verbose logging | `false` |

## 📁 Output Structure

BUTTCRACK produces a single timestamped archive preserving the original directory structure of all collected paths:

```
# Archive written to output directory:
/tmp/buttcrack_logs_2025-11-12_14-30-00.tar.gz

# Contents of the archive (original paths preserved):
buttcrack_logs_2025-11-12_14-30-00/
├── var/
│   ├── lib/
│   │   └── rancher/
│   │       └── rke2/
│   │           └── server/
│   │               └── db/
│   │                   └── etcd/
│   │                       └── member/
│   │                           ├── snap/
│   │                           └── wal/
│   └── log/
│       └── pods/
│           └── ...
└── collection-summary.yaml
```

The `_logs_` infix in the archive name is intentional — it allows the existing `nessie_upload_logs.yaml` Ansible playbook in stack-validation to pick up BUTTCRACK archives with its `*logs*.tar.gz` glob without any changes.

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                             INPUT                               │
│                                                                 │
│   CLI flags                       Env vars                      │
│   --paths /var/lib/etcd,...        BC_PATHS              │
│   --output /tmp                    BC_OUTPUT             │
└─────────────────────────────┬───────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                          BUTTCRACK                              │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │ main.rs                                                   │  │
│  │ parse args + env vars · orchestrate · logging             │  │
│  └───────────────────────────┬───────────────────────────────┘  │
│                              │                                  │
│  ┌───────────────────────────▼───────────────────────────────┐  │
│  │ collector.rs                                              │  │
│  │ validate paths exist · walk each directory recursively    │  │
│  │ build ordered file list                                   │  │
│  └───────────────────────────┬───────────────────────────────┘  │
│                              │                                  │
│  ┌───────────────────────────▼───────────────────────────────┐  │
│  │ archive.rs                                                │  │
│  │ stream files into GzEncoder → tar::Builder                │  │
│  │ write buttcrack_logs_<timestamp>.tar.gz to output dir     │  │
│  └───────────────────────────┬───────────────────────────────┘  │
│                              │                                  │
└─────────────────────────────┼───────────────────────────────────┘
                              │
                              ▼
                  buttcrack_logs_<timestamp>.tar.gz
                              │
┌─────────────────────────────┼───────────────────────────────────┐
│              ANSIBLE / stack-validation                         │
│                             │                                   │
│  ┌───────────────────────────▼───────────────────────────────┐  │
│  │ buttcrack_collect.yaml                                    │  │
│  │ podman run --privileged                                   │  │
│  │   -v /host/path:/host/path:ro   (per input path)         │  │
│  │   -v /tmp:/tmp                  (output, rw)             │  │
│  │                                                           │  │
│  │ sed rename →                                             │  │
│  │   buttcrack_<CLUSTER><CLUSTER_SUFFIX>_logs_<timestamp>    │  │
│  └───────────────────────────┬───────────────────────────────┘  │
│                              │                                  │
│  ┌───────────────────────────▼───────────────────────────────┐  │
│  │ nessie_upload_logs.yaml (reused as-is)                    │  │
│  │ glob: *logs*.tar.gz · WebDAV PUT                          │  │
│  └───────────────────────────┬───────────────────────────────┘  │
│                              │                                  │
└─────────────────────────────┼───────────────────────────────────┘
                              │
                              ▼
                       WebDAV server
                  /pipelines/<id>/logs/
```

## 💡 Common Use Cases

### Collect etcd Database for Investigation
```bash
podman run --rm --privileged \
           -v /var/lib/rancher/rke2/server/db/etcd:/var/lib/rancher/rke2/server/db/etcd:ro \
           -v /tmp:/tmp \
           buttcrack:latest \
           --paths /var/lib/rancher/rke2/server/db/etcd --output /tmp
```

### Collect from k3s Instead
```bash
podman run --rm --privileged \
           -v /var/lib/rancher/k3s/server/db/etcd:/var/lib/rancher/k3s/server/db/etcd:ro \
           -v /tmp:/tmp \
           buttcrack:latest \
           --paths /var/lib/rancher/k3s/server/db/etcd --output /tmp
```

### Collect Multiple Diagnostic Paths
```bash
podman run --rm --privileged \
           -v /var/lib/rancher/rke2/server/db/etcd:/var/lib/rancher/rke2/server/db/etcd:ro \
           -v /var/log/pods:/var/log/pods:ro \
           -v /tmp:/tmp \
           buttcrack:latest \
           --paths /var/lib/rancher/rke2/server/db/etcd,/var/log/pods \
           --output /tmp --verbose
```

## 🔧 Development

### Building from Source

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Format code
cargo fmt

# Check for issues
cargo clippy
```

### Project Structure

```
src/
├── main.rs        # 🚪 CLI interface and orchestration
├── collector.rs   # 📂 Path validation and recursive directory walking
└── archive.rs     # 🗜️ tar.gz archive creation
```

## 🐛 Troubleshooting

**🚫 "Path does not exist or is not accessible"**
- Verify the path exists on the host: `ls -la /your/path`
- Ensure the path is bind-mounted into the container with the same name
- Check that the source path is not empty

**📁 "Permission denied" on output**
- Ensure the output directory is mounted with write access (no `:ro`)
- Try using `/tmp` as the output directory

**🔒 SELinux volume mount errors**
- Add `:Z` to volume mounts on SELinux-enforcing systems:
  ```bash
  -v /your/path:/your/path:ro,Z
  -v /tmp:/tmp:Z
  ```

**📦 Archive not picked up by upload playbook**
- Verify the archive name contains `_logs_` — this is required by the `nessie_upload_logs.yaml` glob
- Check the output directory matches `log_source_dir` passed to the upload playbook

## 📄 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](../LICENSE) file for details.

This project is part of the SUSE Edge Support Tools collection.

## 🙏 Acknowledgments

- 🦀 Built with **Rust** for performance and safety
- 🔌 Designed to integrate natively with **stack-validation** Ansible pipelines
- 🍑 Named after a perfectly reasonable acronym, we promise

---

**Made with ❤️ and 🦀 by the SUSE Support Team**

*Don't let your cluster data slip through the cracks!* 🍑✨
