# Summer of-Bitcoin

Welcome to my Summer of Bitcoin assignment repository! This project documents my work and progress through the assigned tasks over four weeks. Each week focuses on a different aspect of Bitcoin development, from interacting with a Bitcoin node to exploring descriptor wallets. This README provides an overview of the tasks and the work completed.

## Project Overview
This repository contains code, documentation, and resources related to the following weekly assignments:
- **Week 1**: Interacting with a Bitcoin Node
- **Week 2**: Building a P2SH-P2WSH Multisig Transaction
- **Week 3**: Mining a Block
- **Week 4**: Descriptor Wallets

The goal of this project is to deepen my understanding of Bitcoin's technical underpinnings and demonstrate practical skills in blockchain development.

## Weekly Breakdown

### Week 1: Interacting with a Bitcoin Node
- **Objective**: Learn how to connect to and interact with a Bitcoin node using RPC calls or other methods.
- **Tasks**:
  - Set up a Bitcoin node (e.g., Bitcoin Core).
  - Query blockchain data (e.g., block height, transactions).
  - Test basic commands like `getblockchaininfo` or `getrawtransaction`.
- **Files**: [Summer of Bitcoin Week 1](https://github.com/ayushshrivastv/Summer-of-Bitcoin/tree/main/Summer%20of%20Bitcoin%20Week1)
- **Notes**: Successfully connected to a regtest node and retrieved block data.

### Week 2: Building a P2SH-P2WSH Multisig Tx
- **Objective**: Create a Pay-to-Script-Hash (P2SH) transaction with a Pay-to-Witness-Script-Hash (P2WSH) multisig script.
- **Tasks**:
  - Generate a multisig address.
  - Construct and sign a P2SH-P2WSH transaction.
  - Test the transaction on a testnet.
- **Files**: [Summer of Bitcoin Week 2](https://github.com/ayushshrivastv/Summer-of-Bitcoin/tree/main/Summer%20of%20Bitcoin%20Week2)
- **Notes**: Learned the importance of script versioning and witness data.

### Week 3: Mining a Block
- **Objective**: Simulate or perform the process of mining a Bitcoin block.
- **Tasks**:
  - Implement a basic mining algorithm (e.g., proof-of-work).
  - Generate a valid block hash meeting the target difficulty.
  - Integrate with a testnet or regtest environment.
- **Files**: [Summer of Bitcoin Week 3](https://github.com/ayushshrivastv/Summer-of-Bitcoin/tree/main/Summer%20of%20Bitcoin%20Week3)
- **Notes**: Adjusted difficulty for regtest to simulate mining successfully.

### Week 4: Descriptor Wallets
- **Objective**: Explore and implement Bitcoin descriptor wallets.
- **Tasks**:
  - Understand wallet descriptors and their syntax.
  - Create a descriptor wallet and generate addresses.
  - Test sending/receiving transactions with the wallet.
- **Files**: [Summer of Bitcoin Week 4](https://github.com/ayushshrivastv/Summer-of-Bitcoin/tree/main/Summer%20of%20Bitcoin%20Week4)
- **Notes**: Explored the flexibility of descriptors over legacy wallets.

## Prerequisites
To run or replicate the work in this repository, you’ll need:
- [Bitcoin Core](https://bitcoin.org/en/bitcoin-core/) (v24.0 or later)
- [Rust](https://www.rust-lang.org/) (1.60+ recommended) with Cargo
- Testnet access (optional but recommended)
- Git



