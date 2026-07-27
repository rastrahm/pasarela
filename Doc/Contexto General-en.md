# Master Prompt: Agnostic Payment and Settlement System (Web2/Web3)

**Instruction for the LLM/Dev:**

Act as a Principal Fintech and Blockchain Software Architect. We will design and build step by step a payment processor with a microservices architecture in Rust and smart contracts on Solana.

**Main feature:** The system must be mutable and interactive with multiple settlement rails (Multi-Rail), allowing card purchases where the backing funds and final settlement can switch between:

- **Traditional Banking** (Fiat/USD/EUR).
- **Custodial CEX** (Binance API / Custodial Wallet).
- **Non-Custodial On-Chain** (Solana Program / Anchor).

We will develop the project module by module. Do not advance to the next step until I give confirmation.

---

## 🧩 PHASE 1: Domain Design and Rail Abstraction Layer (Payment Rails)

**Objective:** Design the base architecture in Rust, defining the interfaces/traits that enable mutability of the funding provider and settlement.

**Design pattern:** Implement the Strategy pattern in Rust to decouple the funding source (`FundingSource`) from the destination/settlement (`SettlementRail`).

**Required deliverables:**

- Rust trait definition: `trait PaymentProcessor` and `trait LiquidityEngine`.
- Data structures (structs and enums) for `PaymentRequest`, `CardPayload`, `FundingType::[TraditionalBank, BinanceCex, SolanaWallet]`, and `TransactionStatus`.
- Dynamic routing logic (Rail Switcher): a decision engine that chooses the rail according to business rules (e.g., availability, fees, user preference).

---

## 🧩 PHASE 2: Off-Chain Authorization Oracle (Rust Microservice)

**Objective:** Build the microservice that simulates the processing network (Visa/Mastercard) and executes real-time validation ($1–3$ seconds).

**Responsibilities:**

- Card number validation using the Luhn algorithm and brand detection (Visa/Mastercard/Amex).
- Authentication with private key (`X-API-KEY`) to communicate exclusively with the API Gateway.
- Funds and limit evaluation according to the active rail:
  - **If Traditional:** Evaluates static limit or fictitious bank balance.
  - **If Binance:** Queries Spot balance via simulated API and calculates the Hold in USDC/USDT with protection margin (spread buffer).
  - **If Solana:** Queries the On-Chain wallet balance via Solana RPC.

**Required deliverables:**

- Functional HTTP microservice code in Rust (using Axum or Actix-Web).
- Card validation logic module and availability calculation.

---

## 🧩 PHASE 3: On-Chain Settlement Engine (Solana Program with Anchor)

**Objective:** Develop the Solana Smart Contract that acts as the immutable settlement ledger when the selected rail is Web3.

**Responsibilities:**

- Create the instruction `process_payment(amount, brand_code, settlement_rail_id)`.
- Atomic transfer of SPL tokens (USDC/SOL) between the payer/custody account and the merchant account.
- Emit the `PaymentProcessed` event for public audit without exposing sensitive information (Zero PII).

**Required deliverables:**

- Code in Rust using the Anchor framework.
- Account structures and security validators (`#[derive(Accounts)]`).
- Integration tests in TypeScript/Rust to simulate transaction execution.

---

## 🧩 PHASE 4: API Gateway and Orchestration (Rust)

**Objective:** Build the main backend that connects the client request with the Oracle and executes settlement on the appropriate rail.

**Orchestrator flow:**

1. Receives the checkout request: `POST /api/v1/checkout`.
2. Queries the Oracle to authorize the preventive balance Hold.
3. If approved, executes the Settlement Engine:
   - **If Rail == Solana:** Calls the Anchor program using `solana-client`.
   - **If Rail == Binance:** Simulated invocation of the Binance API to debit the custodial balance.
   - **If Rail == Banking:** Generation of a simulated clearing file (ISO 20022 / ACH).
4. Returns confirmation to the client with the transaction ID or Blockchain Hash.

**Required deliverables:**

- Gateway server in Rust.
- Error mapping and structured HTTP responses.

---

## 🧩 PHASE 5: Presentation Layer (React Dashboard & Checkout)

**Objective:** Develop the interactive web interface that demonstrates system mutability.

**UI components:**

- **Card Form:** Entry of fictitious card data.
- **Rail Mutator Selector (Rail Selector):** A dropdown/switch where the user or merchant chooses the settlement method (Traditional Bank, Binance Account, Solana Wallet).
- **Response Viewer:** Displays the transaction log in real time, indicating whether the banking network or the Solana Hash (Tx Signature) was used.

**Required deliverables:**

- Frontend code in React with Tailwind CSS.
- Connection via fetch/axios to the Backend API Gateway.
