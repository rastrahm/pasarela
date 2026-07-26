import type { CheckoutResponse } from '../schemas/gateway'
import {
  getRailLabel,
  getSettlementProofLabel,
  getStatusLabel,
  type TransactionLogEntry,
} from '../schemas/transaction'

/**
 * Props de {@link TransactionViewer}.
 */
export interface TransactionViewerProps {
  /** Entradas del log en orden cronológico. */
  entries: TransactionLogEntry[]
  /** Resultado final del checkout (`CheckoutResponse`), si ya está disponible. */
  result?: CheckoutResponse | null
  /** Mensaje cuando aún no hay entradas en el log. */
  emptyMessage?: string
}

const LEVEL_STYLES: Record<TransactionLogEntry['level'], string> = {
  info: 'border-slate-700 text-slate-200',
  success: 'border-emerald-800/70 text-emerald-200',
  warning: 'border-amber-800/70 text-amber-200',
  error: 'border-rose-800/70 text-rose-200',
}

/** Formatea timestamp del log para la UI (locale es-AR). */
function formatTime(date: Date): string {
  return date.toLocaleTimeString('es-AR', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}

/**
 * Visor de log en tiempo real del checkout — validación, hold, riel y comprobante (UC-10).
 *
 * @param props - {@link TransactionViewerProps}
 * @returns Sección con lista `aria-live` y panel de comprobante opcional.
 */
export function TransactionViewer({
  entries,
  result = null,
  emptyMessage = 'El log aparecerá al iniciar el checkout.',
}: TransactionViewerProps) {
  return (
    <section
      aria-label="Log de transacción"
      className="space-y-3 rounded-xl border border-slate-800 bg-slate-950/50 p-4"
    >
      <h2 className="text-lg font-medium text-white">Transacción</h2>

      {entries.length === 0 ? (
        <p className="text-sm text-slate-500">{emptyMessage}</p>
      ) : (
        <ol
          aria-live="polite"
          className="max-h-56 space-y-2 overflow-y-auto pr-1"
        >
          {entries.map((entry) => (
            <li
              className={`rounded-lg border px-3 py-2 text-sm ${LEVEL_STYLES[entry.level]}`}
              key={entry.id}
            >
              <div className="flex items-start justify-between gap-3">
                <span>{entry.message}</span>
                <time
                  className="shrink-0 text-xs text-slate-500"
                  dateTime={entry.timestamp.toISOString()}
                >
                  {formatTime(entry.timestamp)}
                </time>
              </div>
            </li>
          ))}
        </ol>
      )}

      {result ? (
        <div className="rounded-lg border border-emerald-800/60 bg-emerald-950/30 px-3 py-3 text-sm text-emerald-100">
          <p className="font-medium">Comprobante</p>
          <dl className="mt-2 space-y-1">
            <div className="flex justify-between gap-4">
              <dt className="text-slate-400">ID</dt>
              <dd className="font-mono text-xs">{result.transaction_id}</dd>
            </div>
            <div className="flex justify-between gap-4">
              <dt className="text-slate-400">Estado</dt>
              <dd>{getStatusLabel(result.status)}</dd>
            </div>
            <div className="flex justify-between gap-4">
              <dt className="text-slate-400">Riel usado</dt>
              <dd>{getRailLabel(result.rail_used)}</dd>
            </div>
            {result.settlement_proof ? (
              <div className="flex flex-col gap-1 pt-1">
                <dt className="text-slate-400">
                  {getSettlementProofLabel(result.rail_used)}
                </dt>
                <dd className="break-all font-mono text-xs text-emerald-200">
                  {result.settlement_proof}
                </dd>
              </div>
            ) : null}
          </dl>
        </div>
      ) : null}
    </section>
  )
}
