import { useCallback, useState } from 'react'

import {
  createLogEntry,
  type LogLevel,
  type TransactionLogEntry,
} from '../schemas/transaction'

/**
 * Hook para acumular entradas del log de checkout en tiempo real.
 * CheckoutPage lo usará al orquestar llamadas al Gateway (paso 5.6).
 */
export function useTransactionLog(initialEntries: TransactionLogEntry[] = []) {
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
