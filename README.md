# Jupiter Swap Program

## Overview
The **Jupiter Swap Program** is a Solana-based smart contract that swaps SOL into multiple meme tokens using the **Jupiter Aggregator**. It ensures secure and efficient transactions by leveraging wrapped SOL (wSOL) for compatibility with Jupiter's token swap mechanism.

## Features
- **Dynamic Token Swaps:** Supports flexible selection of meme tokens without hardcoding addresses.
- **Efficient wSOL Handling:** Automatically creates and closes wSOL accounts as needed.
- **Secure Execution:** Implements error handling and access control for robust functionality.
- **Uses Jupiter Aggregator:** Ensures optimal swap routes and liquidity.

## How It Works
- **Swap Execution:** The program splits SOL into three equal parts and swaps each portion into a different meme token via Jupiter.
- **Wrapped SOL Handling:** Since Jupiter only swaps tokens, the program first wraps SOL into wSOL.
- **Jupiter Swap Execution:** The swap is performed dynamically using remaining accounts to allow arbitrary meme token selections.
- **Token Transfer:** Once the swap is completed, the acquired tokens are transferred to the user’s token account.
- **wSOL Account Closure:** The temporary wSOL account is closed to optimize storage and gas costs.


## Installation & Setup

### Prerequisites
- **Rust & Cargo:** Install via [rustup](https://rustup.rs/).
- **Anchor Framework:** Install using:
  ```sh
  cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked
- **Anchor Commands:** 
  ```sh
  anchor build 
  acnhor test --skip-deploy

