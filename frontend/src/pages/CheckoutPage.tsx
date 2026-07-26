import { useCallback, useState } from 'react'

import {
  GatewayApiError,
  mapLocalCheckoutError,
  mapNetworkErrorToUx,
  type CheckoutErrorUx,
} from '../api/errors'
import { submitCheckout } from '../api/gateway'
import { CardForm } from '../components/CardForm'
import { CheckoutErrorAlert } from '../components/CheckoutErrorAlert'
import { RailSelector } from '../components/RailSelector'
import { TransactionViewer } from '../components/TransactionViewer'
import { useTransactionLog } from '../hooks/useTransactionLog'
import { appEnv, isPlaceholderApiKey } from '../config/env'
import { parseCheckoutRequest } from '../schemas/checkout'
import type { CheckoutResponse } from '../schemas/gateway'
import type { CardPayload } from '../schemas/card'
import { DEFAULT_FUNDING_TYPE, type FundingType } from '../schemas/funding'
import {
  getRailLabel,
  TRANSACTION_LOG_MESSAGES,
} from '../schemas/transaction'

const DEFAULT_AMOUNT = 100
const DEFAULT_CURRENCY = 'USD'

/**
 * Página de checkout: orquesta tarjeta, riel y visor; invoca solo al Gateway.
 */
export function CheckoutPage() {
  const [amount, setAmount] = useState(String(DEFAULT_AMOUNT))
  const [currency] = useState(DEFAULT_CURRENCY)
  const [card, setCard] = useState<CardPayload | null>(null)
  const [fundingType, setFundingType] = useState<FundingType>(DEFAULT_FUNDING_TYPE)
  const [checkoutResult, setCheckoutResult] = useState<CheckoutResponse | null>(null)
  const [checkoutError, setCheckoutError] = useState<CheckoutErrorUx | null>(null)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const { entries, append, clear } = useTransactionLog()

  const handleCheckout = useCallback(async () => {
    if (!card) {
      setCheckoutError(
        mapLocalCheckoutError(
          'Validá la tarjeta antes de confirmar el pago.',
          'Completá el formulario y presioná «Validar tarjeta».',
        ),
      )
      return
    }

    const parsedAmount = Number.parseFloat(amount)
    if (Number.isNaN(parsedAmount) || parsedAmount <= 0) {
      setCheckoutError(
        mapLocalCheckoutError(
          'Ingresá un monto válido mayor a 0.',
          'El monto debe ser un número positivo en USD.',
        ),
      )
      return
    }

    let request
    try {
      request = parseCheckoutRequest({
        amount: parsedAmount,
        currency,
        funding_type: fundingType,
        card,
      })
    } catch (error) {
      setCheckoutError(
        mapLocalCheckoutError(
          error instanceof Error ? error.message : 'Datos de checkout inválidos.',
        ),
      )
      return
    }

    clear()
    setCheckoutResult(null)
    setCheckoutError(null)
    setIsSubmitting(true)

    append(TRANSACTION_LOG_MESSAGES.validatingCard)
    append(TRANSACTION_LOG_MESSAGES.authorizing)

    try {
      const result = await submitCheckout(request)

      append(TRANSACTION_LOG_MESSAGES.holdConfirmed, 'success')
      append(
        TRANSACTION_LOG_MESSAGES.settling(getRailLabel(result.rail_used)),
      )

      if (result.status === 'settled') {
        append(TRANSACTION_LOG_MESSAGES.settled, 'success')
      }

      setCheckoutResult(result)
    } catch (error) {
      const ux =
        error instanceof GatewayApiError
          ? error.ux
          : mapNetworkErrorToUx(error)

      append(ux.message, 'error')
      append(TRANSACTION_LOG_MESSAGES.failed, 'error')
      setCheckoutError(ux)
    } finally {
      setIsSubmitting(false)
    }
  }, [amount, append, card, clear, currency, fundingType])

  return (
    <main className="flex min-h-screen flex-col items-center justify-center px-6 py-10">
      <div className="w-full max-w-lg rounded-2xl border border-slate-800 bg-slate-900/60 p-8 shadow-xl">
        <p className="text-sm font-medium uppercase tracking-widest text-emerald-400">
          Pasarela Multi-Rail
        </p>
        <h1 className="mt-2 text-3xl font-semibold text-white">Checkout</h1>

        <CheckoutErrorAlert error={checkoutError} />

        <div className="mt-6 space-y-8">
          <section className="space-y-3">
            <h2 className="text-lg font-medium text-white">Monto</h2>
            <div className="grid grid-cols-[1fr_auto] gap-3">
              <label className="sr-only" htmlFor="amount">
                Monto
              </label>
              <input
                className="rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-white outline-none ring-emerald-500/40 focus:ring-2 disabled:cursor-not-allowed disabled:opacity-50"
                disabled={isSubmitting}
                id="amount"
                inputMode="decimal"
                min="0.01"
                onChange={(event) => setAmount(event.target.value)}
                step="0.01"
                type="number"
                value={amount}
              />
              <span className="flex items-center rounded-lg border border-slate-700 px-3 text-sm text-slate-300">
                {currency}
              </span>
            </div>
          </section>

          <CardForm disabled={isSubmitting} onSubmit={setCard} />
          <RailSelector
            disabled={isSubmitting}
            onChange={setFundingType}
            value={fundingType}
          />
          <TransactionViewer entries={entries} result={checkoutResult} />
        </div>

        <button
          className="mt-6 w-full rounded-lg bg-emerald-500 px-4 py-2.5 font-medium text-slate-950 transition hover:bg-emerald-400 disabled:cursor-not-allowed disabled:opacity-50"
          disabled={isSubmitting || !card}
          onClick={() => {
            void handleCheckout()
          }}
          type="button"
        >
          {isSubmitting ? 'Procesando pago…' : 'Confirmar pago'}
        </button>

        {card ? (
          <p className="mt-4 text-center text-xs text-slate-500">
            Tarjeta lista · terminada en {card.pan.slice(-4)} · riel{' '}
            {fundingType.replace(/_/g, ' ')}
          </p>
        ) : (
          <p className="mt-4 text-center text-xs text-slate-500">
            Validá la tarjeta para habilitar el pago.
          </p>
        )}

        {import.meta.env.DEV &&
        (appEnv.usingDefaults.apiBaseUrl || isPlaceholderApiKey()) ? (
          <p className="mt-3 text-center text-xs text-slate-600">
            Config por defecto ({appEnv.apiBaseUrl}). Copiá{' '}
            <span className="font-mono">.env.example</span> →{' '}
            <span className="font-mono">.env.local</span> si el Gateway usa otros
            valores.
          </p>
        ) : null}
      </div>
    </main>
  )
}
