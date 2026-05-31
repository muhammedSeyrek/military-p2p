# Military P2P Communication Network

A decentralized, peer-to-peer communication system designed for secure, tamper-proof message distribution across independent nodes. Built in Rust, the system utilizes asynchronous networking, cryptographic verification (Merkle Trees, RSA, AES), and a distributed architecture to eliminate single points of failure.

## Project Structure

The workspace is divided into several crates to separate concerns:

* **`crates/mp-crypto`**: Cryptographic primitives, key generation, and hash verification.
* **`crates/mp-protocol`**: Serialization, deserialization, and protocol definitions for P2P messaging.
* **`crates/mp-storage`**: Database schemas, migrations, and query logic (PostgreSQL).
* **`crates/mp-network`**: Asynchronous TCP networking layer using Tokio.
* **`crates/mp-node-general`**: The general headquarters node responsible for dispatching initial operations.
* **`crates/mp-node-commander`**: The individual commander nodes that receive, verify, and store message fragments.

## Development Environment Setup

This project is fully containerized. The recommended workflow is using VS Code with the Dev Containers extension.

1. Open the repository in VS Code.
2. When prompted, click **Reopen in Container** (or run `Dev Containers: Reopen in Container` from the Command Palette).
3. The dev container will build, and all necessary PostgreSQL databases (`pg-general`, `pg-mehmet`, `pg-ali`, etc.) will start automatically as defined in the `docker-compose.yml`.

Once the container is ready, open the integrated terminal. You should see the workspace prompt. Start by compiling the workspace:

```bash
vscode ➜ /workspace $ cargo build --release

## Running the Network

There are two primary ways to run and test the cluster: using the provided orchestration script for local debugging, or spinning up the fully containerized nodes via Docker Compose.

### Option A: Using the Orchestration Script

For local testing and simulation, use the `demo.sh` script. This script manages database resets, key generation, and background node execution.

First, initialize the databases and load the commander configurations:
```bash
vscode ➜ /workspace $ ./demo.sh setup


Start the 7 commander servers in the background:

Bash
vscode ➜ /workspace $ ./demo.sh start
Check the health status of all running nodes:

Bash
vscode ➜ /workspace $ ./demo.sh status
Dispatch a test operation from the General HQ to the network:

Bash
vscode ➜ /workspace $ ./demo.sh dispatch
Read the decrypted message fragment from a specific commander's perspective:

Bash
vscode ➜ /workspace $ ./demo.sh read aylin
To run a complete, automated end-to-end test (including tampering detection):

Bash
vscode ➜ /workspace $ ./demo.sh full-demo
Option B: Running Nodes via Docker Compose
If you want to run the commander nodes purely via Docker (simulating a true distributed environment without local binary execution), the docker-compose.yml includes a specific nodes profile.

To build the optimized release images and start all commander nodes (node-mehmet, node-ali, etc.):

Bash
vscode ➜ /workspace $ docker compose --profile nodes up --build -d
To view the logs of the containerized nodes:

Bash
vscode ➜ /workspace $ docker compose --profile nodes logs -f
To shut down the nodes and the databases:

Bash
vscode ➜ /workspace $ docker compose --profile nodes down
