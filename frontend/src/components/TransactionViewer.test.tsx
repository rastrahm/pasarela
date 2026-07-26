import { describe, expect, it, beforeEach, vi } from 'vitest'

import {
  buildCheckoutLogEntries,
  createLogEntry,
  resetLogSequenceForTests,
  type CheckoutResult,
} from '../schemas/transaction'
import { TransactionViewer } from './TransactionViewer'
import { renderUi, screen } from '../test/test-utils'

const SETTLED_RESULT: CheckoutResult = {
  transaction_id: 'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
  status: 'settled',
  rail_used: 'traditional_bank',
  settlement_proof: 'ACH-MEM-20260726-001',
}

describe('TransactionViewer', () => {
  beforeEach(() => {
    resetLogSequenceForTests()
    vi.useRealTimers()
  })

  it('muestra mensaje vacío sin entradas', () => {
    renderUi(<TransactionViewer entries={[]} />)

    expect(screen.getByText(/el log aparecerá/i)).toBeInTheDocument()
  })

  it('renderiza entradas del log en orden', () => {
    const entries = [
      createLogEntry('Validando tarjeta…'),
      createLogEntry('Hold confirmado', 'success'),
    ]

    renderUi(<TransactionViewer entries={entries} />)

    expect(screen.getByText('Validando tarjeta…')).toBeInTheDocument()
    expect(screen.getByText('Hold confirmado')).toBeInTheDocument()
  })

  it('muestra comprobante con riel y referencia bancaria', () => {
    renderUi(
      <TransactionViewer
        entries={buildCheckoutLogEntries(SETTLED_RESULT)}
        result={SETTLED_RESULT}
      />,
    )

    expect(screen.getByText('Comprobante')).toBeInTheDocument()
    expect(screen.getByText('Liquidada')).toBeInTheDocument()
    expect(screen.getByText('Banco tradicional')).toBeInTheDocument()
    expect(screen.getByText('Referencia bancaria')).toBeInTheDocument()
    expect(screen.getByText('ACH-MEM-20260726-001')).toBeInTheDocument()
  })

  it('muestra Tx Signature para Solana', () => {
    const solanaResult: CheckoutResult = {
      ...SETTLED_RESULT,
      rail_used: 'solana_wallet',
      settlement_proof: 'SOL-MEM-5xK9abc',
    }

    renderUi(
      <TransactionViewer
        entries={buildCheckoutLogEntries(solanaResult)}
        result={solanaResult}
      />,
    )

    expect(screen.getByText('Tx Signature')).toBeInTheDocument()
    expect(screen.getByText('SOL-MEM-5xK9abc')).toBeInTheDocument()
  })

  it('expone región live para actualizaciones', () => {
    renderUi(
      <TransactionViewer entries={[createLogEntry('Autorizando con Oracle…')]} />,
    )

    expect(screen.getByRole('list')).toHaveAttribute('aria-live', 'polite')
  })
})
