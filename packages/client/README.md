# CRISP - Collusion-Resistant Impartial Selection Protocol

CRISP (Collusion-Resistant Impartial Selection Protocol) is a groundbreaking privacy tool for secure, encrypted voting. Built with advanced encryption enabled by the Enclave protocol (FHE, ZKP, threshold cryptography), CRISP ensures verifiable results while maintaining voter confidentiality, shielding governance and decision-making systems against collusion and tampering.

## Why CRISP?

In a digital age marked by increasing concerns over data privacy, security, and computational integrity, ensuring the integrity and confidentiality of voting systems is crucial. CRISP provides a secure, innovative solution that protects against collusion, mitigates governance vulnerabilities, and preserves voter confidentiality, which is essential for trustworthy governance and decision-making processes in both traditional and blockchain-based systems.

## Who is CRISP for?

CRISP is designed for web2 and web3 organizations that require secure and private voting mechanisms. This includes onchain entities (e.g. DAOs), corporations, government bodies, nonprofits, educational institutions, and any other organization that seeks to enhance their decision-making systems with robust, verifiable, and confidential voting protocols.

## Project Structure

```
CRISP/packages
├── /client/
│   ├── /libs/wasm/pkg/ - WebAssembly library package
│   ├── /public/ - Static files
│   ├── /src/ - React components and source code
│   └── [configuration files and README]
├── /evm/ - Ethereum Virtual Machine related code
├── /rust/ - Rust server-side logic
└── /web-rust/ - Rust to WebAssembly logic
```

## Architecture
<p align="center">
<img width="607" alt="image" src="https://github.com/gnosisguild/CRISP/assets/19823989/c8881fe2-1e66-4d99-9347-24e4edc91516">Ï
</p>


## Installation

To set up the CRISP dApp in your local environment, follow these steps:

1. Clone the repository:
   ```sh
   git clone https://github.com/gnosisguild/CRISP.git
   ```
2. Navigate to the `client` directory:
   ```sh
   cd CRISP/packages/client
   ```
3. Install dependencies:
   ```sh
   yarn install
   ```
4. Start the development server:
   ```sh
   yarn dev
   ```

## Setting Up the Enclave Server

### Prerequisites

Before running the Enclave server, you must have Rust installed on your machine along with the necessary environment variables set. Follow these steps to set up your environment:

1. Install Rust:
   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
   This will install `rustup`, Rust's toolchain installer. Follow the on-screen instructions to complete the installation.

2. Add the Rust toolchain to your system's PATH:
   ```sh
   source $HOME/.cargo/env
   ```

### Running the Enclave Server

Navigate to the `enclave_server` directory and run the server with:

```sh
cargo run --bin enclave_server
```

### Setting Up Cipher Clients

Open 4 separate terminal windows for the various components.

1. In two terminals, start the cipher clients by running:
   ```sh
   cargo run --bin start_cipher_client
   ```
   Wait for the `enclave_server` to be up and running before executing this command in each terminal.

2. In the last terminal, run the CLI client:
   ```sh
   cargo run --bin cli
   ```

### Interacting with CRISP via CLI

Once the CLI client is running, you can interact with the CRISP voting protocol as follows:

1. Select the option `CRISP: Voting Protocol (ETH)`.

2. To start a new CRISP round, select the option `Initialize new CRISP round`.

Ensure all components are running and communicating with each other properly before initializing a new round.

Remember to provide exact paths if the `enclave_server`, `start_cipher_client`, and `cli` binaries are located in specific directories. If any specific configuration is needed in the environment files, make sure to outline what changes need to be made and where the files are located. It's also crucial to include any ports that need to be open or additional services that are required for the application to run correctly.

## Contributing

We welcome contributions from the community. Please read our contributing guide and code of conduct before submitting any pull requests or issues.


