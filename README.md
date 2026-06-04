# Escrow Contract (Atomic Swap) on Solana

## Overview

This project implements a decentralized Escrow Contract for secure atomic token swaps on the Solana blockchain using the Anchor Framework.

The escrow enables two parties to exchange SPL tokens without requiring trust between them.

Participants:

* **Maker** creates a trade offer.
* **Taker** accepts the offer and completes the exchange.

The contract leverages:

* Anchor Framework
* Program Derived Addresses (PDAs)
* SPL Token Program
* Vault Accounts for secure asset custody

---

## Why Escrow?

A direct token exchange between two users creates a trust problem:

* The Maker could send tokens first and never receive payment.
* The Taker could do the same.

An escrow contract removes this risk by ensuring that:

* both transfers happen successfully, or
* neither transfer happens at all.

This property is known as an **atomic swap**.

---

# Architecture

## Escrow PDA

Stores the trade agreement and swap parameters.

Seeds:

```rust
[
    b"escrow",
    maker,
    mint_a,
    mint_b,
    seed
]
```

Stored data:

```rust
pub struct Escrow {
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub receive: u64,
    pub bump: u8,
    pub seed: u64,
}
```

### Fields

| Field   | Description                |
| ------- | -------------------------- |
| maker   | Creator of the escrow      |
| mint_a  | Token deposited by Maker   |
| mint_b  | Token requested by Maker   |
| receive | Amount of token B expected |
| bump    | PDA bump seed              |
| seed    | Unique escrow identifier   |

---

## Vault PDA

A token account controlled by the program.

Seeds:

```rust
[
    b"vault",
    maker,
    mint_a,
    seed
]
```

Responsibilities:

* Holds Maker's deposited tokens.
* Prevents unauthorized withdrawals.
* Can only be accessed through program instructions.

Because the Vault is a PDA, no private key exists for it.

---

# Workflow

## 1. Make

The Maker creates a new escrow offer.

Instruction:

```rust
make(seed, deposit, receive)
```

Parameters:

| Parameter | Description                 |
| --------- | --------------------------- |
| seed      | Unique escrow identifier    |
| deposit   | Amount of token A deposited |
| receive   | Amount of token B requested |

Actions performed:

1. Creates the Escrow PDA.
2. Creates the Vault PDA.
3. Transfers token A from Maker into the Vault.

Flow:

```text
Maker Token Account
          |
          v
        Vault
```

Result:

* Tokens are locked inside the Vault.
* Escrow parameters are stored on-chain.

---

## 2. Take

A Taker accepts the offer.

Instruction:

```rust
take()
```

### Step 1

Transfer token B from Taker to Maker.

```text
Taker ---- Token B ----> Maker
```

Amount:

```rust
escrow.receive
```

### Step 2

Transfer token A from Vault to Taker.

```text
Vault ---- Token A ----> Taker
```

Amount:

```rust
vault.amount
```

Result:

```text
Maker receives Token B
Taker receives Token A
```

The swap is executed atomically.

If any transfer fails, the entire transaction is reverted.

---

## 3. Refund

If no Taker accepts the offer, the Maker can reclaim the deposited tokens.

Instruction:

```rust
refund()
```

Actions:

1. Program signs using Vault PDA seeds.
2. Tokens are transferred back to Maker.

Flow:

```text
Vault
  |
  v
Maker
```

Result:

* Funds are returned safely.
* No third party can interfere.

---

# PDA Structure

```text
                    +------------------+
                    |      Maker       |
                    +------------------+
                             |
                             |
                             v
                 +----------------------+
                 |     Escrow PDA       |
                 +----------------------+
                             |
                             |
                             v
                 +----------------------+
                 |      Vault PDA       |
                 +----------------------+
                             |
                     Holds Token A
                             |
                             v

                 +----------------------+
                 |       Taker          |
                 +----------------------+
```

---

# Security Considerations

## Program Controlled Vault

The Vault account is owned by a PDA.

```rust
token::authority = vault
```

Since PDAs have no private keys, only the program can authorize token movements.

---

## Atomic Transactions

The `take()` instruction performs both transfers within the same transaction.

Properties:

* No partial execution.
* No risk of one-sided settlement.
* Automatic rollback on failure.

---

## Escrow Isolation

Each escrow instance is uniquely identified by:

```rust
seed
```

This allows multiple independent escrows to coexist without conflicts.

---

# Example Scenario

Maker creates an offer:

```text
Deposit:
100 USDC

Request:
2 SOL
```

Escrow state:

```text
Vault:
100 USDC

Expected:
2 SOL
```

Taker accepts:

```text
Taker -> Maker:
2 SOL

Vault -> Taker:
100 USDC
```

Final balances:

```text
Maker:
+2 SOL

Taker:
+100 USDC
```

---

# Technologies Used

* Rust
* Solana
* Anchor Framework
* SPL Token Program
* Program Derived Addresses (PDAs)
* Cross Program Invocations (CPIs)

---

# Key Concepts Demonstrated

This project showcases:

* PDA account design
* Secure token custody
* Vault pattern implementation
* Anchor account constraints
* SPL token transfers
* Cross Program Invocations (CPIs)
* Atomic swap mechanisms
* Solana account architecture
* On-chain state management

---

# Future Improvements

Potential production-ready enhancements:

### Account Cleanup

* Close Escrow account after successful swap.
* Close Vault account after token withdrawal.

### Validation

* Additional amount verification.
* Custom error handling with `require!`.

### Events

```rust
#[event]
pub struct EscrowCreated {}

#[event]
pub struct EscrowTaken {}

#[event]
pub struct EscrowRefunded {}
```

### Testing

* Unit tests
* Integration tests
* Local validator testing
* Edge-case coverage

---

# Conclusion

The Escrow Contract is a fundamental DeFi building block that demonstrates how to securely exchange assets on Solana without requiring trust between participants.

The project combines:

* PDA-based account ownership
* Secure token vaults
* CPI interactions
* Atomic settlement guarantees

These patterns are commonly used in decentralized exchanges (DEXs), OTC trading systems, marketplaces, and broader DeFi infrastructure.
