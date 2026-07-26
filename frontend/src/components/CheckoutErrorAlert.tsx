import type { CheckoutErrorUx } from '../api/errors'

export interface CheckoutErrorAlertProps {
  /** Error estructurado a mostrar; `null` oculta el componente. */
  error: CheckoutErrorUx | null
}

const KIND_STYLES: Record<CheckoutErrorUx['kind'], string> = {
  insufficient_funds: 'border-amber-800/70 bg-amber-950/40 text-amber-100',
  invalid_card: 'border-rose-800/70 bg-rose-950/40 text-rose-100',
  invalid_request: 'border-rose-800/70 bg-rose-950/40 text-rose-100',
  rail_unavailable: 'border-amber-800/70 bg-amber-950/40 text-amber-100',
  unauthorized: 'border-rose-800/70 bg-rose-950/40 text-rose-100',
  conflict: 'border-slate-700 bg-slate-900/60 text-slate-200',
  network: 'border-sky-800/70 bg-sky-950/40 text-sky-100',
  unknown: 'border-rose-800/70 bg-rose-950/40 text-rose-100',
}

/**
 * Alerta accesible con título, mensaje e hint según el tipo de error del Gateway.
 */
export function CheckoutErrorAlert({ error }: CheckoutErrorAlertProps) {
  if (!error) {
    return null
  }

  const alertId = `checkout-error-${error.kind}`

  return (
    <div
      aria-labelledby={alertId}
      className={`mt-4 rounded-lg border px-4 py-3 text-sm ${KIND_STYLES[error.kind]}`}
      role="alert"
    >
      <p className="font-medium" id={alertId}>
        {error.title}
      </p>
      <p className="mt-1">{error.message}</p>
      {error.hint ? (
        <p className="mt-2 text-xs opacity-90">{error.hint}</p>
      ) : null}
      {error.status ? (
        <p className="mt-2 font-mono text-xs opacity-60">
          HTTP {error.status}
          {error.errorCode ? ` · ${error.errorCode}` : ''}
        </p>
      ) : null}
    </div>
  )
}
