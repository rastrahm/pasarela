# Acta de Cierre — Fase 0 (Planificación)

> Gate **0.8** · Validación y aprobación de documentación  
> Fecha: **2026-07-25**  
> Proyecto: Pasarela Multi-Rail (Web2/Web3)

---

## 1. Declaración

Se declara **cerrada la Fase 0 — Planificación** del procesador de pagos multi-rail, con autorización para iniciar la **Fase 1 — Dominio y abstracción de rieles**.

La planificación define el alcance MVP, la arquitectura de microservicios, los casos de uso, el modelo de datos, los flujos operativos, las decisiones de diseño D1–D12 y la hoja de ruta hasta producción.

---

## 2. Entregables verificados

| Entregable | Ubicación | Estado |
|------------|-----------|--------|
| Contexto y alcance MVP | [Contexto General.md](./Contexto%20General.md) | ✅ |
| Arquitectura del sistema | [Arquitectura.md](./Arquitectura.md) | ✅ |
| Casos de uso, ER y flujos | [Casos-de-Uso-ER-Flujos.md](./Casos-de-Uso-ER-Flujos.md) | ✅ |
| Plan de implementación | [Plan-de-Implementacion.md](./Plan-de-Implementacion.md) | ✅ |
| Decisiones D1–D12 | [Arquitectura §12](./Arquitectura.md#12-decisiones-de-diseño--resueltas-fase-0) | ✅ |
| Esqueleto Oracle (Axum, auth, Luhn) | `oracle/` | ✅ (15 tests verdes) |
| Directivas de código | `*.cursorrules` | ✅ |

---

## 3. Checklist de coherencia documental

Revisión cruzada entre Arquitectura, Casos de Uso y Plan:

| # | Criterio | Resultado |
|---|----------|-----------|
| 1 | Tres rieles definidos (TraditionalBank, BinanceCex, SolanaWallet) | ✅ Coherente en los 3 docs |
| 2 | Oracle independiente en `oracle/` (monorepo D5) | ✅ Coherente |
| 3 | Antifraude externo `antifraud/` (D11) | ✅ Arquitectura + UC-12 + flujos |
| 4 | Framework Axum (D1) | ✅ Coherente |
| 5 | Fallback automático por prioridad (D3) | ✅ Rail Switcher + UC-08 |
| 6 | Spread Binance configurable (D4) | ✅ Env + UC-04 |
| 7 | Token PAN en memoria (D6) | ✅ Oracle + UC-03 |
| 8 | mTLS postergado a Fase 7/8 (D7) | ✅ Coherente |
| 9 | 3DS fuera de MVP (D8) | ✅ Coherente |
| 10 | Idempotency-Key en Fase 4 (D9) | ✅ UC-01 + Gateway |
| 11 | Commitment Solana `finalized` (D10) | ✅ UC-07 + §4.4.3 |
| 12 | API key comercio (D12) | ✅ MERCHANT ER + UC-01 |
| 13 | Frontera: Frontend nunca → Oracle | ✅ Coherente |
| 14 | Zero PII on-chain | ✅ PaymentProcessed |
| 15 | Fases secuenciales con gates | ✅ Plan §2 y §4–12 |

**Resultado:** documentación **coherente** entre sí. No se detectaron contradicciones bloqueantes.

---

## 4. Decisiones de diseño registradas (resumen)

Ver tabla completa en [Arquitectura §12](./Arquitectura.md#12-decisiones-de-diseño--resueltas-fase-0).

| ID | Resolución |
|----|------------|
| D1 | Axum (Gateway + Oracle + Antifraude) |
| D2 | Local validator (dev) + devnet (CI) |
| D3 | Fallback automático por prioridad |
| D4 | `BINANCE_SPREAD_BUFFER_PCT` configurable |
| D5 | Monorepo |
| D6 | Token hash en memoria; PAN descartado |
| D7 | mTLS en Fase 7/8 |
| D8 | 3DS post-MVP |
| D9 | Idempotency-Key en Fase 4 |
| D10 | Commitment `finalized` |
| D11 | Servicio `antifraud/` simulado |
| D12 | API key por comercio en Fase 4 |

---

## 5. Alcance MVP acordado

### Incluido

- Procesador de pagos con tarjeta ficticia (Luhn)
- Tres rieles de liquidación intercambiables
- Oracle de autorización aislado
- Servicio antifraude simulado
- Programa Anchor Solana (devnet/local)
- API Gateway con orquestación
- Frontend checkout demo
- Seguridad fail closed, sin PII on-chain

### Excluido (post-MVP)

- 3-D Secure / PSD2 SCA
- Integración adquirente real (Stripe, Adyen)
- Token vault HSM / PCI nivel 1
- KYC/AML real
- Solana mainnet sin auditoría
- mTLS (hasta staging)

---

## 6. Verificación técnica pre-Fase 1

| Verificación | Comando / acción | Resultado |
|--------------|------------------|-----------|
| Oracle compila y tests pasan | `cd oracle && cargo test` | ✅ 15 tests OK |
| Workspace pasarela | — | ⬜ Pendiente (Fase 1) |
| Programa Anchor | — | ⬜ Pendiente (Fase 3) |

---

## 7. Autorización

| Rol | Acción | Fecha |
|-----|--------|-------|
| Stakeholder / Producto | Aprobación gate Fase 0 → inicio Fase 1 | 2026-07-25 |
| Arquitectura | Documentación validada (checklist §3) | 2026-07-25 |

**Próximo paso autorizado:** Fase 1 — crear workspace Cargo en `pasarela/` con crates `domain` y `rail-switcher`.

---

## Referencias

- [Plan-de-Implementacion.md §4](./Plan-de-Implementacion.md#4-fase-0--planificación-fase-actual)
- [Arquitectura.md](./Arquitectura.md)
- [Casos-de-Uso-ER-Flujos.md](./Casos-de-Uso-ER-Flujos.md)
