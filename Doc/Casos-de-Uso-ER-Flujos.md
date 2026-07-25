# Casos de Uso, Modelo Entidad-Relación y Flujos

> Documento complementario de [Arquitectura.md](./Arquitectura.md). Describe el comportamiento funcional, el modelo de datos y los flujos operativos del sistema de pagos multi-rail.

---

## 1. Actores del Sistema

| Actor | Tipo | Descripción |
|-------|------|-------------|
| **Comprador** | Humano | Usuario que ingresa datos de tarjeta y confirma el pago en el checkout. |
| **Comercio** | Humano / Sistema | Entidad que recibe el pago; puede configurar preferencia de riel por defecto. |
| **Frontend (Checkout)** | Sistema | Interfaz React que captura datos, valida con Zod y muestra el resultado. |
| **API Gateway** | Sistema | Orquestador principal; expone `POST /api/v1/checkout` y coordina Oracle + Settlement. |
| **Oracle de Autorización** | Sistema independiente | Servicio aislado en `oracle/`; simula red procesadora; valida tarjeta y evalúa fondos (~1–3 s). Solo accesible desde red interna por el Gateway. |
| **Rail Switcher** | Sistema | Motor de decisión que selecciona el riel activo según reglas de negocio. |
| **Settlement Engine** | Sistema | Ejecuta la liquidación en el riel elegido. |
| **Riel Tradicional** | Sistema externo | Genera compensación bancaria simulada (ISO 20022 / ACH). |
| **Riel Binance CEX** | Sistema externo | API simulada de Binance para débito custodial Spot. |
| **Riel Solana** | Sistema externo | Programa Anchor + RPC Solana para transferencia SPL on-chain. |

### Diagrama de actores y zonas de despliegue

```mermaid
flowchart LR
    subgraph Public["Zona pública — pasarela/"]
        Comprador --> Frontend
        Comercio --> Frontend
        Frontend --> Gateway["API Gateway"]
        Gateway --> Switcher["Rail Switcher"]
        Gateway --> Settlement["Settlement Engine"]
        Settlement --> Bank["Riel Tradicional"]
        Settlement --> Binance["Riel Binance CEX"]
        Settlement --> Solana["Riel Solana"]
    end

    subgraph Private["Zona privada — oracle/"]
        Oracle["Oracle de Autorización"]
    end

    Gateway -->|"HTTPS + X-API-KEY\n(allowlist)"| Oracle
    Oracle --> Bank
    Oracle --> Binance
    Oracle --> Solana
```

> **Regla de frontera:** el Frontend y el Comercio **nunca** invocan al Oracle. Toda comunicación pasa por el Gateway mediante el crate `oracle-client`.

---

## 2. Casos de Uso

### 2.1 Diagrama general

```mermaid
flowchart TB
    subgraph Actores
        C[Comprador]
        M[Comercio]
    end

    subgraph Pasarela["Pasarela — pasarela/"]
        UC01["UC-01: Realizar checkout"]
        UC02["UC-02: Seleccionar riel"]
        UC05["UC-05: Liquidar — Banco"]
        UC06["UC-06: Liquidar — Binance"]
        UC07["UC-07: Liquidar — Solana"]
        UC08["UC-08: Fallback de riel"]
        UC09["UC-09: Consultar transacción"]
        UC10["UC-10: Visualizar log"]
    end

    subgraph OracleSvc["Oracle — oracle/"]
        UC03["UC-03: Validar tarjeta"]
        UC04["UC-04: Autorizar hold"]
        UC11["UC-11: Control de acceso"]
    end

    C --> UC01
    C --> UC02
    C --> UC10
    M --> UC02
    UC01 -.->|vía Gateway| UC03
    UC01 -.->|vía Gateway| UC04
    UC03 --> UC11
    UC04 --> UC11
    UC01 --> UC05
    UC01 --> UC06
    UC01 --> UC07
    UC01 --> UC08
    UC01 --> UC09
    UC01 --> UC10
```

---

### UC-01: Realizar checkout con tarjeta

| Campo | Detalle |
|-------|---------|
| **ID** | UC-01 |
| **Actor principal** | Comprador |
| **Actores secundarios** | Frontend, API Gateway, Oracle, Settlement Engine |
| **Descripción** | El comprador ingresa datos ficticios de tarjeta, selecciona un riel de liquidación y confirma el pago. El sistema autoriza, retiene fondos y liquida en el riel activo. |
| **Precondiciones** | Frontend disponible; Gateway y Oracle operativos; al menos un riel configurado y habilitado. |
| **Postcondiciones (éxito)** | Transacción en estado `Settled`; comprobante generado (Tx Signature, ID CEX o ref. bancaria). |
| **Postcondiciones (fallo)** | Transacción en estado `Failed`; hold liberado si existía. |

**Flujo principal:**

1. El comprador ingresa monto, datos de tarjeta y selecciona riel (UC-02).
2. El frontend valida localmente con Zod.
3. El frontend envía `POST /api/v1/checkout`.
4. El Gateway invoca UC-03 (validar tarjeta) vía `oracle-client` → `POST /internal/v1/authorize`.
5. El Gateway invoca UC-04 (autorizar hold) en la misma llamada al Oracle (UC-11 valida acceso antes de procesar).
6. El Gateway ejecuta liquidación según riel: UC-05, UC-06 o UC-07.
7. El Gateway responde con `transaction_id`, `status` y `settlement_proof`.
8. El frontend muestra el resultado en el visor de transacciones (UC-10).

**Flujos alternativos:**

| ID | Condición | Acción |
|----|-----------|--------|
| 1a | Validación Zod falla en frontend | Mostrar errores de campo; no enviar request. |
| 4a | Luhn inválido o marca no reconocida | Oracle rechaza → Gateway responde `422`. |
| 5a | Fondos insuficientes en riel seleccionado | Oracle rechaza → Gateway responde `402`; evaluar UC-08 si fallback habilitado. |
| 6a | Riel no disponible (timeout RPC/API) | Gateway responde `503`; evaluar UC-08. |
| 6b | Error interno en liquidación | Gateway responde `500`; transacción → `Failed`. |

---

### UC-02: Seleccionar riel de liquidación

| Campo | Detalle |
|-------|---------|
| **ID** | UC-02 |
| **Actor principal** | Comprador / Comercio |
| **Descripción** | El usuario elige explícitamente el método de liquidación: Banco Tradicional, Binance Account o Solana Wallet. |
| **Precondiciones** | Componente `RailSelector` renderizado; rieles habilitados en configuración. |
| **Postcondiciones** | `FundingType` queda asociado al `PaymentRequest` enviado al Gateway. |

**Flujo principal:**

1. El usuario abre el selector de riel en el checkout.
2. El sistema muestra opciones disponibles con descripción breve.
3. El usuario selecciona un riel.
4. El frontend incluye `funding_type` en el payload del checkout.

**Flujos alternativos:**

| ID | Condición | Acción |
|----|-----------|--------|
| 3a | Usuario no selecciona riel | Usar riel por defecto del comercio o `TraditionalBank`. |
| 3b | Riel seleccionado deshabilitado | Mostrar aviso; impedir submit hasta elegir otro. |

---

### UC-03: Validar tarjeta (Luhn + marca)

| Campo | Detalle |
|-------|---------|
| **ID** | UC-03 |
| **Actor principal** | Oracle de Autorización |
| **Descripción** | Valida el PAN con algoritmo de Luhn y detecta la marca (Visa, Mastercard, Amex). |
| **Precondiciones** | Request desde red interna; UC-11 aprobado (`X-API-KEY` + IP en allowlist). |
| **Ubicación** | Servicio independiente en `oracle/src/validation/` |
| **Postcondiciones (éxito)** | Marca detectada; PAN tokenizado en memoria; validación registrada en audit log. |
| **Postcondiciones (fallo)** | Rechazo con código `INVALID_CARD`. |

**Flujo principal:**

1. Oracle recibe `CardPayload` (PAN, expiry, CVV ficticio) en `POST /internal/v1/authorize`.
2. UC-11 valida autenticación y origen (fail closed).
3. Ejecuta algoritmo de Luhn sobre el PAN.
4. Detecta marca según prefijos (4=Visa, 51–55=MC, 34/37=Amex).
5. Tokeniza PAN en memoria; descarta PAN completo antes de responder.
6. Retorna `CardValidationResult { valid: true, brand, brand_code }`.

**Flujos alternativos:**

| ID | Condición | Acción |
|----|-----------|--------|
| 2a | UC-11 rechaza (API Key, IP, rate limit) | `401` / `403` / `429`; no procesar tarjeta. |
| 3a | Luhn falla | Retornar `valid: false`, error `INVALID_CARD`. |
| 4a | Marca no reconocida | Retornar `valid: false`, error `UNKNOWN_BRAND`. |

---

### UC-04: Autorizar hold de fondos

| Campo | Detalle |
|-------|---------|
| **ID** | UC-04 |
| **Actor principal** | Oracle de Autorización |
| **Descripción** | Evalúa disponibilidad de fondos en el riel activo y crea un hold preventivo sobre el monto solicitado. |
| **Precondiciones** | UC-03 completado con éxito; UC-11 aprobado; riel seleccionado y habilitado. |
| **Postcondiciones (éxito)** | Hold creado con `HoldId`; fondos reservados temporalmente en persistencia del Oracle. |
| **Postcondiciones (fallo)** | Sin hold; fondos no modificados. |
| **Ubicación** | Servicio independiente en `oracle/src/funds/` |

**Flujo principal:**

1. Oracle recibe monto, moneda y `FundingType` (misma request de autorización).
2. Consulta saldo/límite según riel:
   - **TraditionalBank**: límite estático o saldo bancario ficticio.
   - **BinanceCex**: saldo Spot simulado − spread buffer.
   - **SolanaWallet**: balance SPL vía RPC.
3. Si fondos ≥ monto + fees, crea hold y retorna `HoldId`.
4. Registra hold en persistencia del Oracle (`oracle/` — entidad `HOLD`).
5. Registra evento en `ORACLE_AUDIT_LOG` sin PII.

**Flujos alternativos:**

| ID | Condición | Acción |
|----|-----------|--------|
| 3a | Fondos insuficientes | Retornar error `INSUFFICIENT_FUNDS`. |
| 2a | RPC/API del riel no responde | Retornar error `RAIL_UNAVAILABLE`. |
| 2b | Spread buffer deja saldo por debajo del monto (Binance) | Retornar `INSUFFICIENT_FUNDS`. |

---

### UC-11: Control de acceso al Oracle

| Campo | Detalle |
|-------|---------|
| **ID** | UC-11 |
| **Actor principal** | Oracle de Autorización |
| **Actores secundarios** | API Gateway (único caller autorizado) |
| **Descripción** | Valida que cada request al Oracle provenga del Gateway autorizado, en red interna, con credenciales válidas y dentro de límites de tasa. Replica el control de acceso entre adquirente y procesador de tarjetas (CDE PCI-DSS). Ver [Arquitectura.md §9](./Arquitectura.md#9-seguridad). |
| **Precondiciones** | Oracle desplegado en red privada; `ORACLE_API_KEY` y `ORACLE_ALLOWED_CALLERS` configurados. |
| **Postcondiciones (éxito)** | Request pasa al handler de negocio (UC-03 / UC-04). |
| **Postcondiciones (fallo)** | Respuesta inmediata de error; **fail closed** — no se procesa tarjeta ni fondos. |
| **Ubicación** | `oracle/src/auth/` |

**Flujo principal:**

1. Request llega a ruta `/internal/v1/*`.
2. Middleware verifica header `X-API-KEY` contra `ORACLE_API_KEY`.
3. Middleware verifica IP/host del caller contra `ORACLE_ALLOWED_CALLERS`.
4. Middleware verifica rate limit por caller.
5. Si todo OK, delega al handler; si no, rechaza.

**Flujos alternativos:**

| ID | Condición | Acción |
|----|-----------|--------|
| 2a | API Key ausente o incorrecta | `401 Unauthorized`. |
| 3a | IP no está en allowlist | `403 Forbidden`. |
| 4a | Rate limit excedido | `429 Too Many Requests`. |
| 1a | Ruta fuera de `/internal/v1/` (excepto `/health`) | `404 Not Found`. |

---

### UC-05: Liquidar pago — Riel Tradicional

| Campo | Detalle |
|-------|---------|
| **ID** | UC-05 |
| **Actor principal** | Settlement Engine |
| **Descripción** | Genera archivo de compensación bancaria simulado (ISO 20022 / ACH) y confirma el asentamiento fiat. |
| **Precondiciones** | Hold activo (UC-04); `FundingType = TraditionalBank`. |
| **Postcondiciones** | Settlement con referencia bancaria; hold consumido; transacción → `Settled`. |

**Flujo principal:**

1. Settlement Engine recibe `hold_id` y datos de la transacción.
2. Genera archivo de compensación simulado con monto, moneda y referencia.
3. Marca hold como `Consumed`.
4. Persiste `SettlementReceipt { type: Bank, reference_id }`.
5. Actualiza transacción a `Settled`.

---

### UC-06: Liquidar pago — Riel Binance CEX

| Campo | Detalle |
|-------|---------|
| **ID** | UC-06 |
| **Actor principal** | Settlement Engine |
| **Descripción** | Debita saldo custodial Spot (USDC/USDT) mediante API simulada de Binance. |
| **Precondiciones** | Hold activo; `FundingType = BinanceCex`. |
| **Postcondiciones** | Settlement con `cex_order_id`; hold consumido; transacción → `Settled`. |

**Flujo principal:**

1. Settlement Engine invoca API simulada de Binance con monto y par USDC/USDT.
2. API retorna confirmación de débito con `order_id`.
3. Hold → `Consumed`; transacción → `Settled`.
4. Retorna `SettlementReceipt { type: BinanceCex, order_id }`.

**Flujos alternativos:**

| ID | Condición | Acción |
|----|-----------|--------|
| 1a | API Binance timeout | Liberar hold; transacción → `Failed`; error `503`. |
| 1b | Saldo cambió entre hold y settle | Rechazar settle; liberar hold; error `402`. |

---

### UC-07: Liquidar pago — Riel Solana (On-Chain)

| Campo | Detalle |
|-------|---------|
| **ID** | UC-07 |
| **Actor principal** | Settlement Engine |
| **Descripción** | Invoca `process_payment` en el programa Anchor para transferir tokens SPL al comercio. |
| **Precondiciones** | Hold activo; `FundingType = SolanaWallet`; wallet con SOL para fees. |
| **Postcondiciones** | Tx confirmada on-chain; evento `PaymentProcessed` emitido; transacción → `Settled`. |

**Flujo principal:**

1. Settlement Engine construye transacción con `solana-client`.
2. Invoca `process_payment(amount, brand_code, settlement_rail_id)`.
3. Programa transfiere tokens SPL de pagador a comercio.
4. Emite evento `PaymentProcessed` (sin PII).
5. Espera confirmación de la red.
6. Persiste `SettlementReceipt { type: Solana, tx_signature }`.
7. Hold → `Consumed`; transacción → `Settled`.

**Flujos alternativos:**

| ID | Condición | Acción |
|----|-----------|--------|
| 2a | Cuenta sin signer válido | Tx rechazada; hold liberado; error on-chain. |
| 2b | Integer overflow en amount | Programa retorna error; hold liberado. |
| 5a | Tx no confirmada en timeout | Transacción → `Pending`; reintento configurable. |

---

### UC-08: Conmutar riel automáticamente (Fallback)

| Campo | Detalle |
|-------|---------|
| **ID** | UC-08 |
| **Actor principal** | Rail Switcher |
| **Descripción** | Si el riel preferido no está disponible o tiene fondos insuficientes, selecciona el siguiente riel según prioridad configurada. |
| **Precondiciones** | Fallback habilitado en configuración; existe al menos un riel alternativo habilitado. |
| **Postcondiciones** | Nuevo riel seleccionado y flujo de autorización reintentado; o rechazo final si ningún riel es viable. |

**Flujo principal:**

1. Rail Switcher recibe fallo del riel preferido (fondos, timeout, deshabilitado).
2. Consulta lista de rieles ordenados por prioridad.
3. Para cada candidato: consulta disponibilidad vía Oracle.
4. Selecciona el primer riel viable.
5. Reintenta UC-04 y liquidación con el nuevo riel.

**Flujos alternativos:**

| ID | Condición | Acción |
|----|-----------|--------|
| 4a | Ningún riel viable | Rechazar checkout; transacción → `Failed`. |

---

### UC-09: Consultar estado de transacción

| Campo | Detalle |
|-------|---------|
| **ID** | UC-09 |
| **Actor principal** | Comprador / Comercio |
| **Descripción** | Consulta el estado actual de una transacción por su `transaction_id`. |
| **Precondiciones** | Transacción existente en persistencia. |
| **Postcondiciones** | Retorna `TransactionStatus`, riel usado y prueba de asentamiento si aplica. |

**Flujo principal:**

1. Cliente envía `GET /api/v1/transactions/{id}`.
2. Gateway consulta repositorio de transacciones.
3. Retorna DTO con status, rail, proof y timestamps.

---

### UC-10: Visualizar log de transacción

| Campo | Detalle |
|-------|---------|
| **ID** | UC-10 |
| **Actor principal** | Comprador |
| **Descripción** | Muestra en tiempo real el progreso del checkout: validación, hold, riel usado y comprobante final. |
| **Precondiciones** | Checkout iniciado. |
| **Postcondiciones** | Log visible con cada etapa del flujo documentada. |

**Flujo principal:**

1. Al enviar checkout, `TransactionViewer` muestra "Validando tarjeta…".
2. Tras respuesta parcial o final, actualiza: marca detectada, riel seleccionado, hold OK/FAIL.
3. Muestra comprobante: Tx Signature (Solana), order ID (Binance) o ref. bancaria.

---

## 3. Diagramas Entidad-Relación

Los modelos off-chain se **particionan por servicio**: el Gateway (`pasarela/`) gestiona transacciones y settlements; el Oracle (`oracle/`) gestiona holds y auditoría de autorización. Se comunican por `hold_id` vía API, no por FK directa en base de datos compartida.

### 3.1 Modelo off-chain — Pasarela (`pasarela/`)

Entidades gestionadas por el API Gateway y el Settlement Engine.

```mermaid
erDiagram
    MERCHANT ||--o{ TRANSACTION : "recibe"
    MERCHANT ||--o| RAIL_PREFERENCE : "tiene default"
    TRANSACTION ||--|| PAYMENT_REQUEST : "origina"
    TRANSACTION ||--o| SETTLEMENT : "liquida"
    TRANSACTION ||--o{ GATEWAY_AUDIT_LOG : "audita"
    PAYMENT_REQUEST ||--|| CARD_TOKEN : "referencia"
    PAYMENT_REQUEST }o--|| FUNDING_TYPE : "usa"
    SETTLEMENT }o--|| FUNDING_TYPE : "ejecuta en"
    RAIL_CONFIG ||--|| FUNDING_TYPE : "configura"
    TRANSACTION ||--o| ORACLE_HOLD_REF : "referencia hold_id"

    MERCHANT {
        uuid id PK
        string name
        string default_currency
        timestamp created_at
    }

    TRANSACTION {
        uuid id PK
        uuid merchant_id FK
        enum status "Pending|Authorized|Held|Settled|Failed|Reversed"
        decimal amount
        string currency
        enum funding_type
        string settlement_proof
        uuid oracle_hold_id "referencia lógica al Oracle"
        timestamp created_at
        timestamp updated_at
    }

    PAYMENT_REQUEST {
        uuid id PK
        uuid transaction_id FK
        uuid card_token_id FK
        enum funding_type
        decimal amount
        string currency
        json metadata
    }

    CARD_TOKEN {
        uuid id PK
        string token "hash del PAN, nunca PAN completo"
        enum brand "Visa|Mastercard|Amex|Unknown"
        smallint brand_code
        string last_four
        string expiry_month
        string expiry_year
    }

    SETTLEMENT {
        uuid id PK
        uuid transaction_id FK
        enum rail_type
        string proof "tx_signature|order_id|bank_ref"
        enum status "Completed|Failed"
        timestamp settled_at
    }

    ORACLE_HOLD_REF {
        uuid hold_id "ID devuelto por Oracle API"
        uuid transaction_id FK
    }

    FUNDING_TYPE {
        enum value "TraditionalBank|BinanceCex|SolanaWallet"
    }

    RAIL_CONFIG {
        uuid id PK
        enum funding_type
        boolean enabled
        int priority
        decimal fee_percent
        decimal spread_buffer
        json connection_params
    }

    RAIL_PREFERENCE {
        uuid id PK
        uuid merchant_id FK
        enum preferred_rail
        boolean fallback_enabled
    }

    GATEWAY_AUDIT_LOG {
        uuid id PK
        uuid transaction_id FK
        string event_type
        string detail "sin PII"
        timestamp occurred_at
    }
```

---

### 3.2 Modelo off-chain — Oracle (`oracle/`)

Entidades gestionadas exclusivamente por el servicio independiente de autorización.

```mermaid
erDiagram
    HOLD ||--o{ ORACLE_AUDIT_LOG : "audita"
    HOLD }o--|| FUNDING_TYPE : "sobre"
    AUTHORIZATION_REQUEST ||--|| HOLD : "genera"

    AUTHORIZATION_REQUEST {
        uuid id PK
        uuid gateway_request_id "correlación con Gateway"
        enum funding_type
        decimal amount
        string currency
        enum brand "Visa|Mastercard|Amex"
        smallint brand_code
        string card_token_hash "nunca PAN"
        enum result "Approved|Rejected"
        timestamp created_at
    }

    HOLD {
        uuid id PK
        uuid authorization_request_id FK
        enum funding_type
        decimal amount
        enum status "Active|Consumed|Released|Expired"
        timestamp expires_at
        timestamp created_at
    }

    ORACLE_AUDIT_LOG {
        uuid id PK
        uuid hold_id FK
        string event_type
        string detail "sin PII — request_id, riel, resultado"
        string caller_ip "IP del Gateway"
        timestamp occurred_at
    }

    FUNDING_TYPE {
        enum value "TraditionalBank|BinanceCex|SolanaWallet"
    }
```

#### Descripción de entidades por servicio

| Entidad | Servicio | Responsabilidad |
|---------|----------|-----------------|
| **TRANSACTION** | Pasarela | Agregado raíz del ciclo de vida del pago |
| **SETTLEMENT** | Pasarela | Registro del asentamiento con prueba verificable |
| **ORACLE_HOLD_REF** | Pasarela | Referencia lógica al `hold_id` del Oracle (sin FK DB) |
| **HOLD** | Oracle | Reserva temporal de fondos antes de liquidación |
| **AUTHORIZATION_REQUEST** | Oracle | Registro de cada intento de autorización (sin PAN) |
| **ORACLE_AUDIT_LOG** | Oracle | Trazabilidad de auth, holds y rechazos de acceso |
| **GATEWAY_AUDIT_LOG** | Pasarela | Trazabilidad de checkout y settlement |

---

### 3.3 Vista unificada Pasarela ↔ Oracle (referencia lógica)

```mermaid
flowchart LR
    subgraph Pasarela["pasarela/"]
        TX["TRANSACTION"]
        SET["SETTLEMENT"]
    end

    subgraph OracleSvc["oracle/"]
        HOLD["HOLD"]
        AUTH_REQ["AUTHORIZATION_REQUEST"]
    end

    TX -->|"oracle_hold_id vía API"| HOLD
    AUTH_REQ --> HOLD
    SET -->|"POST /internal/v1/hold/release\n(si settlement falla)"| HOLD
```

| Campo Pasarela | Campo Oracle | Mecanismo |
|----------------|--------------|-----------|
| `TRANSACTION.oracle_hold_id` | `HOLD.id` | Respuesta de `POST /internal/v1/authorize` |
| `TRANSACTION.id` | `AUTHORIZATION_REQUEST.gateway_request_id` | Correlación en request |
| `SETTLEMENT.status = Failed` | `HOLD.status → Released` | `POST /internal/v1/hold/release` |

---

### 3.4 Modelo on-chain (programa Anchor — Solana)

Las cuentas on-chain no almacenan PII. El libro mayor inmutable registra montos, códigos de marca y referencias de riel.

```mermaid
erDiagram
    SETTLEMENT_STATE ||--o{ PAYMENT_RECORD : "registra"
    MERCHANT_PUBKEY ||--|| SETTLEMENT_STATE : "identifica"
    PAYER_TOKEN_ACCOUNT ||--o{ PAYMENT_RECORD : "debita"
    MERCHANT_TOKEN_ACCOUNT ||--o{ PAYMENT_RECORD : "acredita"

    SETTLEMENT_STATE {
        pubkey merchant PK "parte de PDA seeds"
        u64 bump
        u64 total_settled
        u64 payment_count
        i64 last_settlement_at
    }

    PAYMENT_RECORD {
        u64 amount
        u8 brand_code "sin PAN"
        u64 settlement_rail_id
        i64 timestamp
        pubkey tx_signer
    }

    PAYER_TOKEN_ACCOUNT {
        pubkey address PK
        pubkey owner
        u64 balance
        pubkey mint "USDC|SOL mint"
    }

    MERCHANT_TOKEN_ACCOUNT {
        pubkey address PK
        pubkey owner
        u64 balance
        pubkey mint
    }

    MERCHANT_PUBKEY {
        pubkey key PK
    }
```

#### Relación PDA

```
Seeds: ["settlement", merchant_pubkey.as_ref()]
Program: payment-settlement
Account: SettlementState (PDA)
```

#### Evento on-chain (no persistido como entidad, emitido en logs)

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `amount` | u64 | Monto transferido en tokens SPL |
| `brand_code` | u8 | Código numérico de marca (Visa=1, MC=2, Amex=3) |
| `settlement_rail_id` | u64 | Identificador del riel |
| `timestamp` | i64 | Unix timestamp del bloque |

---

### 3.5 Vista unificada off-chain ↔ on-chain

```mermaid
flowchart LR
    subgraph OffChain["Persistencia Off-Chain"]
        TX["TRANSACTION"]
        SET["SETTLEMENT"]
    end

    subgraph OnChain["Solana On-Chain"]
        PDA["SettlementState PDA"]
        EVT["Event PaymentProcessed"]
    end

    TX -->|"funding_type = SolanaWallet"| SET
    SET -->|"proof = tx_signature"| EVT
    SET -->|"actualiza contadores"| PDA
```

| Campo off-chain | Equivalente on-chain |
|-----------------|----------------------|
| `SETTLEMENT.proof` | Tx Signature de la transacción Solana |
| `TRANSACTION.amount` | `PaymentProcessed.amount` |
| `CARD_TOKEN.brand_code` | `PaymentProcessed.brand_code` |
| `RAIL_CONFIG.id` | `PaymentProcessed.settlement_rail_id` |

---

## 4. Flujos

### 4.1 Flujo principal de checkout (diagrama de actividad)

```mermaid
flowchart TD
    Start([Comprador inicia checkout]) --> Input[Ingresa tarjeta + monto + riel]
    Input --> Zod{Zod válido?}
    Zod -->|No| ShowErr[Mostrar errores UI]
    ShowErr --> Input
    Zod -->|Sí| Post[POST /api/v1/checkout]
    Post --> Validate[Oracle: Luhn + marca]
    Validate --> ValidOK{Válida?}
    ValidOK -->|No| R422[422 Invalid Card]
    ValidOK -->|Sí| SelectRail[Rail Switcher: confirmar riel]
    SelectRail --> EvalFunds[Oracle: evaluar fondos]
    EvalFunds --> FundsOK{Fondos OK?}
    FundsOK -->|No| Fallback{Fallback habilitado?}
    Fallback -->|Sí| SelectRail
    Fallback -->|No| R402[402 Insufficient Funds]
    FundsOK -->|Sí| CreateHold[Crear Hold]
    CreateHold --> RailType{Tipo de riel}

    RailType -->|TraditionalBank| SettleBank[Generar ISO 20022 / ACH]
    RailType -->|BinanceCex| SettleBinance[Débito API Binance]
    RailType -->|SolanaWallet| SettleSolana[process_payment on-chain]

    SettleBank --> Success[200 OK + proof]
    SettleBinance --> Success
    SettleSolana --> Success

    SettleBank -->|Error| Fail[500 / liberar hold]
    SettleBinance -->|Error| Fail
    SettleSolana -->|Error| Fail

    Success --> ShowLog[TransactionViewer: mostrar comprobante]
    R422 --> ShowLog
    R402 --> ShowLog
    Fail --> ShowLog
    ShowLog --> End([Fin])
```

---

### 4.2 Flujo de autorización del Oracle (secuencia)

```mermaid
sequenceDiagram
    participant GW as API Gateway
    participant O as Oracle (oracle/)
    participant Auth as auth/ middleware
    participant L as validation/
    participant F as funds/
    participant R as Riel Externo

    GW->>O: POST /internal/v1/authorize + X-API-KEY
    O->>Auth: verificar acceso (UC-11)

    alt API Key inválida
        Auth-->>GW: 401 Unauthorized
    else IP no en allowlist
        Auth-->>GW: 403 Forbidden
    else Rate limit excedido
        Auth-->>GW: 429 Too Many Requests
    end

    Auth->>L: validate(pan)
    L-->>O: CardValidationResult (PAN tokenizado)

    alt Tarjeta inválida
        O-->>GW: 422 INVALID_CARD
    end

    O->>F: evaluate_funds(amount, funding_type)

    alt TraditionalBank
        F->>F: consultar límite/saldo ficticio
    else BinanceCex
        F->>R: GET /spot/balance (simulado)
        R-->>F: balance USDC/USDT
        F->>F: aplicar spread buffer
    else SolanaWallet
        F->>R: RPC getTokenAccountBalance
        R-->>F: balance SPL
    end

    F-->>O: FundStatus

    alt Fondos insuficientes
        O-->>GW: 402 INSUFFICIENT_FUNDS
    else Fondos OK
        O->>O: crear Hold + ORACLE_AUDIT_LOG
        O-->>GW: 200 { hold_id, brand, brand_code }
    end
```

---

### 4.3 Flujo de decisión del Rail Switcher

```mermaid
flowchart TD
    Start([PaymentRequest recibido]) --> Pref{Preferencia explícita?}
    Pref -->|Sí| CheckPref[Riel preferido]
    Pref -->|No| MerchantDefault[Riel default del comercio]
    MerchantDefault --> CheckPref
    CheckPref --> Enabled{Habilitado en RAIL_CONFIG?}
    Enabled -->|No| NextRail[Siguiente por prioridad]
    Enabled -->|Sí| Avail[Consultar disponibilidad Oracle]
    Avail --> AvailOK{Disponible y fondos OK?}
    AvailOK -->|Sí| Cost[Evaluar costo/fees]
    AvailOK -->|No| Fallback{Fallback habilitado?}
    Fallback -->|Sí| NextRail
    Fallback -->|No| Reject[Rechazar — sin riel viable]
    Cost --> Selected([Riel seleccionado])
    NextRail --> MoreRails{Hay más rieles?}
    MoreRails -->|Sí| CheckPref
    MoreRails -->|No| Reject
```

---

### 4.4 Flujos de liquidación por riel

#### 4.4.1 Riel Tradicional

```mermaid
sequenceDiagram
    participant SE as Settlement Engine
    participant Bank as Riel Tradicional
    participant DB as Persistencia

    SE->>DB: obtener Hold activo
    SE->>Bank: generar compensación ISO 20022 / ACH
    Bank-->>SE: bank_reference_id
    SE->>DB: Settlement { proof: bank_reference_id }
    SE->>DB: Hold → Consumed
    SE->>DB: Transaction → Settled
```

#### 4.4.2 Riel Binance CEX

```mermaid
sequenceDiagram
    participant SE as Settlement Engine
    participant BX as Binance API (simulada)
    participant DB as Persistencia

    SE->>DB: obtener Hold activo
    SE->>BX: POST /order/debit { amount, pair: USDC }
    BX-->>SE: { order_id, status: filled }
    SE->>DB: Settlement { proof: order_id }
    SE->>DB: Hold → Consumed
    SE->>DB: Transaction → Settled
```

#### 4.4.3 Riel Solana On-Chain

```mermaid
sequenceDiagram
    participant SE as Settlement Engine
    participant SC as solana-client
    participant PRG as Programa Anchor
    participant RPC as Solana RPC
    participant DB as Persistencia

    SE->>DB: obtener Hold activo
    SE->>SC: build process_payment tx
    SC->>PRG: process_payment(amount, brand_code, rail_id)
    PRG->>PRG: transfer SPL tokens
    PRG->>PRG: emit PaymentProcessed
    PRG-->>SC: Ok
    SC->>RPC: sendTransaction + confirm
    RPC-->>SC: tx_signature
    SE->>DB: Settlement { proof: tx_signature }
    SE->>DB: Hold → Consumed
    SE->>DB: Transaction → Settled
```

---

### 4.5 Máquina de estados de la transacción

```mermaid
stateDiagram-v2
    [*] --> Pending : checkout iniciado
    Pending --> Authorized : tarjeta validada (Luhn OK)
    Pending --> Failed : tarjeta inválida / error

    Authorized --> Held : hold creado
    Authorized --> Failed : fondos insuficientes

    Held --> Settled : liquidación exitosa
    Held --> Failed : error en settlement
    Held --> Pending : fallback de riel (reintento)

    Settled --> Reversed : reverso manual (futuro)
    Failed --> [*]
    Settled --> [*]
    Reversed --> [*]
```

| Transición | Disparador | Actor |
|------------|-----------|-------|
| Pending → Authorized | Oracle valida tarjeta | Oracle |
| Authorized → Held | Hold creado con fondos OK | Oracle |
| Held → Settled | Settlement completado | Settlement Engine |
| Held → Failed | Error en riel / timeout | Settlement Engine |
| Held → Pending | Fallback activa nuevo riel | Rail Switcher |
| Settled → Reversed | Reverso administrativo (scope futuro) | Comercio / Admin |

---

### 4.6 Flujo de manejo de errores

```mermaid
flowchart TD
    Err([Error detectado]) --> Type{Tipo de error}

    Type -->|INVALID_CARD| E422[HTTP 422 + código INVALID_CARD]
    Type -->|INSUFFICIENT_FUNDS| E402[HTTP 402 + liberar hold si existe]
    Type -->|RAIL_UNAVAILABLE| E503[HTTP 503 + evaluar fallback]
    Type -->|UNAUTHORIZED| E401[HTTP 401]
    Type -->|INTERNAL| E500[HTTP 500 + liberar hold + log]

    E503 --> FB{Fallback?}
    FB -->|Sí| Retry[Reintentar con siguiente riel]
    FB -->|No| Fail[Transaction → Failed]

    E402 --> FB
    E422 --> Fail
    E401 --> Fail
    E500 --> Fail
    Retry --> OK{Éxito?}
    OK -->|Sí| Done([Continuar flujo normal])
    OK -->|No| Fail
    Fail --> End([Responder al cliente])
    E422 --> End
    Done --> End
```

---

### 4.7 Flujo del frontend (checkout UI)

```mermaid
sequenceDiagram
    participant U as Comprador
    participant CF as CardForm
    participant RS as RailSelector
    participant CP as CheckoutPage
    participant TV as TransactionViewer
    participant GW as API Gateway

    U->>CF: ingresa PAN, expiry, CVV
    U->>RS: selecciona riel
    U->>CP: click "Pagar"
    CP->>CF: validar (Zod)

    alt Validación falla
        CF-->>U: errores inline
    end

    CP->>TV: "Validando tarjeta…"
    CP->>GW: POST /api/v1/checkout

    alt 200 OK
        GW-->>CP: { transaction_id, status, rail, proof }
        CP->>TV: mostrar comprobante (hash/ref)
    else 402 / 422 / 503
        GW-->>CP: { error_code, message }
        CP->>TV: mostrar error + riel intentado
    end

    TV-->>U: log completo visible
```

---

## 5. Matriz Caso de Uso ↔ Entidad ↔ Flujo

| Caso de uso | Servicio | Entidades principales | Flujo de referencia |
|-------------|----------|----------------------|---------------------|
| UC-01 Checkout | Pasarela | TRANSACTION, PAYMENT_REQUEST, SETTLEMENT, ORACLE_HOLD_REF | §4.1, §4.7 |
| UC-02 Seleccionar riel | Pasarela | RAIL_PREFERENCE, FUNDING_TYPE | §4.3 |
| UC-03 Validar tarjeta | Oracle | AUTHORIZATION_REQUEST, ORACLE_AUDIT_LOG | §4.2 |
| UC-04 Autorizar hold | Oracle | HOLD, AUTHORIZATION_REQUEST | §4.2 |
| UC-05 Liquidar Banco | Pasarela | SETTLEMENT, HOLD (release vía API) | §4.4.1 |
| UC-06 Liquidar Binance | Pasarela | SETTLEMENT, HOLD (release vía API) | §4.4.2 |
| UC-07 Liquidar Solana | Pasarela | SETTLEMENT, SETTLEMENT_STATE, PaymentProcessed | §4.4.3 |
| UC-08 Fallback | Pasarela | RAIL_CONFIG, RAIL_PREFERENCE | §4.3, §4.6 |
| UC-09 Consultar tx | Pasarela | TRANSACTION, SETTLEMENT | §4.5 |
| UC-10 Visualizar log | Pasarela | GATEWAY_AUDIT_LOG, TRANSACTION | §4.7 |
| UC-11 Control de acceso | Oracle | ORACLE_AUDIT_LOG | §4.2 |

---

## Referencias

- [Arquitectura.md](./Arquitectura.md) — Componentes, traits, fases e **estructuras de seguridad Web2/Web3** (§9)
- [Contexto General.md](./Contexto%20General.md) — Prompt maestro del proyecto
