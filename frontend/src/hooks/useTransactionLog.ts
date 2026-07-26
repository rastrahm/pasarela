import { useCallback, useState } from 'react'

import {
  createLogEntry,
  type LogLevel,
  type TransactionLogEntry,
} from '../schemas/transaction'

/** API retornada por {@link useTransactionLog}. */
export interface TransactionLogApi {
  /** Entradas acumuladas en orden cronológico. */
  entries: TransactionLogEntry[]
  /** Agrega una línea al log con nivel opcional. */
  append: (message: string, level?: LogLevel) => void
  /** Reemplaza todas las entradas (p. ej. replay de un flujo). */
  replace: (nextEntries: TransactionLogEntry[]) => void
  /** Vacía el log. */
  clear: () => void
}

/**
 * Hook para acumular entradas del log de checkout en tiempo real (UC-10).
 *
 * @param initialEntries - Entradas iniciales del log (default: `[]`).
 * @returns {@link TransactionLogApi} con `entries`, `append`, `replace` y `clear`.
 */
export function useTransactionLog(
  initialEntries: TransactionLogEntry[] = [],
): TransactionLogApi {
  const [entries, setEntries] = useState<TransactionLogEntry[]>(initialEntries)

  const append = useCallback((message: string, level: LogLevel = 'info') => {
    setEntries((current) => [...current, createLogEntry(message, level)])
  }, [])

  const replace = useCallback((nextEntries: TransactionLogEntry[]) => {
    setEntries(nextEntries)
  }, [])

  const clear = useCallback(() => {
    setEntries([])
  }, [])

  return { entries, append, replace, clear }
}
