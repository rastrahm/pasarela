# Documentación — Internacionalización (i18n)

> **Documentation / Documentación:** [Español (es)](Documentation-i18n-es.md) · [English (en)](Documentation-i18n-en.md)

Cada documento existe en tres formas:

| Forma | Propósito |
|-------|-----------|
| `Nombre.md` | Archivo original + banner de idioma (retrocompatible) |
| `Nombre-es.md` | Copia canónica en español |
| `Nombre-en.md` | Traducción al inglés |

## Enlaces rápidos

### Diseño central

| Tema | ES | EN |
|------|----|----|
| Arquitectura | [Arquitectura-es.md](Arquitectura-es.md) | [Arquitectura-en.md](Arquitectura-en.md) |
| Casos de uso y ER | [Casos-de-Uso-ER-Flujos-es.md](Casos-de-Uso-ER-Flujos-es.md) | [Casos-de-Uso-ER-Flujos-en.md](Casos-de-Uso-ER-Flujos-en.md) |
| Plan de implementación | [Plan-de-Implementacion-es.md](Plan-de-Implementacion-es.md) | [Plan-de-Implementacion-en.md](Plan-de-Implementacion-en.md) |
| Contexto maestro | [Contexto General-es.md](Contexto%20General-es.md) | [Contexto General-en.md](Contexto%20General-en.md) |

### Operación

| Tema | ES | EN |
|------|----|----|
| Runbook desarrollo local | [Runbook-Desarrollo-es.md](Runbook-Desarrollo-es.md) | [Runbook-Desarrollo-en.md](Runbook-Desarrollo-en.md) |
| Runbook staging | [Runbook-Staging-es.md](Runbook-Staging-es.md) | [Runbook-Staging-en.md](Runbook-Staging-en.md) |
| Provisión VPS | [Provision-VPS-Fase-7-es.md](Provision-VPS-Fase-7-es.md) | [Provision-VPS-Fase-7-en.md](Provision-VPS-Fase-7-en.md) |
| Guía de pruebas | [Pruebas-es.md](Pruebas-es.md) | [Pruebas-en.md](Pruebas-en.md) |
| CI | [CI-es.md](CI-es.md) | [CI-en.md](CI-en.md) |

### Calidad y deuda

| Tema | ES | EN |
|------|----|----|
| Checklist QA | [Checklist-QA-Fase-6-es.md](Checklist-QA-Fase-6-es.md) | [Checklist-QA-Fase-6-en.md](Checklist-QA-Fase-6-en.md) |
| Revisión seguridad | [Revision-Seguridad-Fase-6-es.md](Revision-Seguridad-Fase-6-es.md) | [Revision-Seguridad-Fase-6-en.md](Revision-Seguridad-Fase-6-en.md) |
| Revisión rendimiento | [Revision-Rendimiento-Fase-6-es.md](Revision-Rendimiento-Fase-6-es.md) | [Revision-Rendimiento-Fase-6-en.md](Revision-Rendimiento-Fase-6-en.md) |
| Deuda técnica | [Deuda-Tecnica-es.md](Deuda-Tecnica-es.md) | [Deuda-Tecnica-en.md](Deuda-Tecnica-en.md) |

### READMEs por servicio

| Servicio | ES | EN |
|----------|----|----|
| Raíz | [README-es.md](../README-es.md) | [README-en.md](../README-en.md) |
| API Gateway | [api-gateway/README-es.md](../crates/api-gateway/README-es.md) | [api-gateway/README-en.md](../crates/api-gateway/README-en.md) |
| Oracle | [oracle/README-es.md](../oracle/README-es.md) | [oracle/README-en.md](../oracle/README-en.md) |
| Antifraude | [antifraud/README-es.md](../antifraud/README-es.md) | [antifraud/README-en.md](../antifraud/README-en.md) |
| Binance sim | [binance-sim/README-es.md](../binance-sim/README-es.md) | [binance-sim/README-en.md](../binance-sim/README-en.md) |
| Frontend | [frontend/README-es.md](../frontend/README-es.md) | [frontend/README-en.md](../frontend/README-en.md) |
| Programa Solana | [payment-settlement/README-es.md](../programs/payment-settlement/README-es.md) | [payment-settlement/README-en.md](../programs/payment-settlement/README-en.md) |
| Deploy staging | [deploy/staging/README-es.md](../deploy/staging/README-es.md) | [deploy/staging/README-en.md](../deploy/staging/README-en.md) |

## Scripts de mantenimiento

```bash
# Regenerar *-es.md desde *.md actuales (tras editar originales)
./scripts/i18n-docs-sync-es.sh

# Añadir/actualizar banners de idioma en originales
./scripts/i18n-docs-add-banners.sh
```

Al actualizar documentación, editá el archivo `-es.md` (o el original) y sincronizá la traducción `-en.md`.
