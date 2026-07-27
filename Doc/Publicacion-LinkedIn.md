> **Documentation / Documentación:** [Español (es)](Publicacion-LinkedIn-es.md) · [English (en)](Publicacion-LinkedIn-en.md)
>
# Publicación LinkedIn — Pasarela Multi-Rail

> Borrador listo para copiar/pegar. Ajustá enlaces al repo público cuando esté disponible.

---

## Versión principal (recomendada)

**Pasarela Multi-Rail: un checkout, tres formas de liquidar**

Terminamos la primera versión funcional de un procesador de pagos que une Web2 y Web3 en un solo flujo de checkout.

El comercio cobra con tarjeta; el sistema elige (o conmuta) el riel de liquidación:

🏦 **Banco tradicional** — compensación fiat simulada  
📊 **Binance CEX** — débito Spot USDC/USDT (API simulada)  
⛓️ **Solana** — saldo SPL vía RPC + programa Anchor en devnet

**Stack**

- Backend en **Rust** (Axum): API Gateway, Oracle de autorización, antifraude y simuladores
- **PostgreSQL** para holds, transacciones e idempotencia
- Frontend **React + TypeScript + Zod** con checkout en vivo
- **Anchor / Solana** — programa desplegado en devnet
- CI con GitHub Actions: Rust, Vitest, Playwright y build Docker staging

**Seguridad by design**

- Oracle en red interna (solo el Gateway lo invoca)
- Fail closed en auth y fondos
- Sin PAN/CVV on-chain ni en logs
- Idempotencia en checkout

Validamos el flujo completo en local y contra **Solana devnet** — smoke tests en los tres rieles.

Próximo paso: staging en VPS con TLS y preparación para producción.

Documentación, arquitectura y runbooks en el repositorio.

#Rust #Solana #Web3 #Payments #Fintech #OpenSource #React #Blockchain #SoftwareArchitecture

---

## Versión corta (alternativa)

Checkout unificado + liquidación multi-rail: banco, CEX o Solana SPL.

Rust · React · Anchor · PostgreSQL · CI completo.

Oracle aislado, idempotencia, tests E2E — validado en local y devnet.

Repo + docs en enlace 👇

#RustLang #Solana #Fintech #Payments

---

## Versión técnica (para audiencia dev)

Publicamos el monorepo **Pasarela Multi-Rail**:

```
Frontend (React) → API Gateway (Axum) → Oracle (holds + fondos)
                      ↓                      ↓
              Settlement adapters      Antifraude / Binance sim
                      ↓
              Solana RPC + program 4cKoe...564B (devnet)
```

- Workspace Cargo: `domain`, `rail-switcher`, `settlement-adapters`, `api-gateway`
- Contrato Gateway ↔ Oracle vía `oracle-client` (`/internal/v1/`)
- Fixtures canónicos + smoke scripts + Docker Compose staging (Caddy TLS)
- Playwright 6/6 en stack local

Fase actual: staging pre-producción. Gate producción con checklist go/no-go.

Enlace al repo: _[completar]_

#Rust #Microservices #Solana #Anchor #TDD

---

## Sugerencias al publicar

1. Adjuntar **captura del checkout** (comprobante con los 3 rieles) o diagrama de arquitectura.
2. Enlace al **README** del repo en el primer comentario si LinkedIn acorta URLs.
3. Etiquetar solo tecnologías que uses realmente en el post.
4. Si el repo es privado al inicio, indicar “documentación y demo disponibles bajo solicitud” hasta el release público.
