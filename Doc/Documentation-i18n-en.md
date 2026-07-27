# Documentation — Internationalization (i18n)

> **Documentation / Documentación:** [Español (es)](Documentation-i18n-es.md) · [English (en)](Documentation-i18n-en.md)

Each document exists in three forms:

| Form | Purpose |
|------|---------|
| `Name.md` | Original file + language banner (backward compatible) |
| `Name-es.md` | Spanish canonical copy |
| `Name-en.md` | English translation |

## Quick links

### Core design

| Topic | ES | EN |
|-------|----|----|
| Architecture | [Arquitectura-es.md](Arquitectura-es.md) | [Arquitectura-en.md](Arquitectura-en.md) |
| Use cases & ER | [Casos-de-Uso-ER-Flujos-es.md](Casos-de-Uso-ER-Flujos-es.md) | [Casos-de-Uso-ER-Flujos-en.md](Casos-de-Uso-ER-Flujos-en.md) |
| Implementation plan | [Plan-de-Implementacion-es.md](Plan-de-Implementacion-es.md) | [Plan-de-Implementacion-en.md](Plan-de-Implementacion-en.md) |
| Master context | [Contexto General-es.md](Contexto%20General-es.md) | [Contexto General-en.md](Contexto%20General-en.md) |

### Operations

| Topic | ES | EN |
|-------|----|----|
| Local dev runbook | [Runbook-Desarrollo-es.md](Runbook-Desarrollo-es.md) | [Runbook-Desarrollo-en.md](Runbook-Desarrollo-en.md) |
| Staging runbook | [Runbook-Staging-es.md](Runbook-Staging-es.md) | [Runbook-Staging-en.md](Runbook-Staging-en.md) |
| VPS provisioning | [Provision-VPS-Fase-7-es.md](Provision-VPS-Fase-7-es.md) | [Provision-VPS-Fase-7-en.md](Provision-VPS-Fase-7-en.md) |
| Testing guide | [Pruebas-es.md](Pruebas-es.md) | [Pruebas-en.md](Pruebas-en.md) |
| CI | [CI-es.md](CI-es.md) | [CI-en.md](CI-en.md) |

### Quality & debt

| Topic | ES | EN |
|-------|----|----|
| QA checklist | [Checklist-QA-Fase-6-es.md](Checklist-QA-Fase-6-es.md) | [Checklist-QA-Fase-6-en.md](Checklist-QA-Fase-6-en.md) |
| Security review | [Revision-Seguridad-Fase-6-es.md](Revision-Seguridad-Fase-6-es.md) | [Revision-Seguridad-Fase-6-en.md](Revision-Seguridad-Fase-6-en.md) |
| Performance review | [Revision-Rendimiento-Fase-6-es.md](Revision-Rendimiento-Fase-6-es.md) | [Revision-Rendimiento-Fase-6-en.md](Revision-Rendimiento-Fase-6-en.md) |
| Technical debt | [Deuda-Tecnica-es.md](Deuda-Tecnica-es.md) | [Deuda-Tecnica-en.md](Deuda-Tecnica-en.md) |

### Service READMEs

| Service | ES | EN |
|---------|----|----|
| Root | [README-es.md](../README-es.md) | [README-en.md](../README-en.md) |
| API Gateway | [api-gateway/README-es.md](../crates/api-gateway/README-es.md) | [api-gateway/README-en.md](../crates/api-gateway/README-en.md) |
| Domain | [domain/README-es.md](../crates/domain/README-es.md) | [domain/README-en.md](../crates/domain/README-en.md) |
| Rail Switcher | [rail-switcher/README-es.md](../crates/rail-switcher/README-es.md) | [rail-switcher/README-en.md](../crates/rail-switcher/README-en.md) |
| Oracle | [oracle/README-es.md](../oracle/README-es.md) | [oracle/README-en.md](../oracle/README-en.md) |
| Antifraud | [antifraud/README-es.md](../antifraud/README-es.md) | [antifraud/README-en.md](../antifraud/README-en.md) |
| Binance sim | [binance-sim/README-es.md](../binance-sim/README-es.md) | [binance-sim/README-en.md](../binance-sim/README-en.md) |
| Frontend | [frontend/README-es.md](../frontend/README-es.md) | [frontend/README-en.md](../frontend/README-en.md) |
| Solana program | [payment-settlement/README-es.md](../programs/payment-settlement/README-es.md) | [payment-settlement/README-en.md](../programs/payment-settlement/README-en.md) |
| Staging deploy | [deploy/staging/README-es.md](../deploy/staging/README-es.md) | [deploy/staging/README-en.md](../deploy/staging/README-en.md) |

## Maintenance scripts

```bash
# Regenerate *-es.md from current *.md (after editing originals)
./scripts/i18n-docs-sync-es.sh

# Add/update language banners on originals
./scripts/i18n-docs-add-banners.sh
```

When updating documentation, edit the `-es.md` file (or original), then sync the English `-en.md` translation.
