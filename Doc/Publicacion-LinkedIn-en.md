# LinkedIn Post — Pasarela Multi-Rail

> Ready-to-copy draft. Adjust links to the public repo when available.

---

## Main version (recommended)

**Pasarela Multi-Rail: one checkout, three settlement paths**

We completed the first functional version of a payment processor that unifies Web2 and Web3 in a single checkout flow.

The merchant charges with a card; the system selects (or switches) the settlement rail:

🏦 **Traditional bank** — simulated fiat settlement  
📊 **Binance CEX** — Spot USDC/USDT debit (simulated API)  
⛓️ **Solana** — SPL balance via RPC + Anchor program on devnet

**Stack**

- **Rust** backend (Axum): API Gateway, authorization Oracle, antifraud, and simulators
- **PostgreSQL** for holds, transactions, and idempotency
- **React + TypeScript + Zod** frontend with live checkout
- **Anchor / Solana** — program deployed on devnet
- CI with GitHub Actions: Rust, Vitest, Playwright, and staging Docker build

**Security by design**

- Oracle on internal network (only Gateway invokes it)
- Fail closed on auth and funds
- No PAN/CVV on-chain or in logs
- Checkout idempotency

We validated the full flow locally and against **Solana devnet** — smoke tests on all three rails.

Next step: VPS staging with TLS and production preparation.

Documentation, architecture, and runbooks in the repository.

#Rust #Solana #Web3 #Payments #Fintech #OpenSource #React #Blockchain #SoftwareArchitecture

---

## Short version (alternative)

Unified checkout + multi-rail settlement: bank, CEX, or Solana SPL.

Rust · React · Anchor · PostgreSQL · full CI.

Isolated Oracle, idempotency, E2E tests — validated locally and on devnet.

Repo + docs at link 👇

#RustLang #Solana #Fintech #Payments

---

## Technical version (for dev audience)

We published the **Pasarela Multi-Rail** monorepo:

```
Frontend (React) → API Gateway (Axum) → Oracle (holds + funds)
                      ↓                      ↓
              Settlement adapters      Antifraud / Binance sim
                      ↓
              Solana RPC + program 4cKoe...564B (devnet)
```

- Cargo workspace: `domain`, `rail-switcher`, `settlement-adapters`, `api-gateway`
- Gateway ↔ Oracle contract via `oracle-client` (`/internal/v1/`)
- Canonical fixtures + smoke scripts + Docker Compose staging (Caddy TLS)
- Playwright 6/6 on local stack

Current phase: pre-production staging. Production gate with go/no-go checklist.

Repo link: _[fill in]_

#Rust #Microservices #Solana #Anchor #TDD

---

## Publishing tips

1. Attach a **checkout screenshot** (receipt with all 3 rails) or architecture diagram.
2. Link to the repo **README** in the first comment if LinkedIn shortens URLs.
3. Tag only technologies you actually use in the post.
4. If the repo is private initially, note "documentation and demo available on request" until public release.
