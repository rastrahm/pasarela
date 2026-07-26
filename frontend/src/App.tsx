import { useCallback, useState } from 'react'

import { CardForm } from './components/CardForm'
import { RailSelector } from './components/RailSelector'
import { TransactionViewer } from './components/TransactionViewer'
import { useTransactionLog } from './hooks/useTransactionLog'
import type { CardPayload } from './schemas/card'
import { DEFAULT_FUNDING_TYPE, type FundingType } from './schemas/funding'
import {
  buildCheckoutLogEntries,
  type CheckoutResult,
} from './schemas/transaction'

const DEMO_PROOFS: Record<FundingType, string> = {
  traditional_bank: 'ACH-MEM-20260726-001',
  binance_cex: 'CEX-MEM-ORD-8842',
  solana_wallet: 'SOL-MEM-5xK9abcDevnetSig',
}

function buildDemoCheckoutResult(rail: FundingType): CheckoutResult {
  return {
    transaction_id: 'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
    status: 'settled',
    rail_used: rail,
    settlement_proof: DEMO_PROOFS[rail],
  }
}

/**
 * Shell inicial de la pasarela. CheckoutPage completo en paso 5.6.
 */
function App() {
  const [validatedCard, setValidatedCard] = useState<CardPayload | null>(null)
  const [fundingType, setFundingType] = useState<FundingType>(DEFAULT_FUNDING_TYPE)
  const [checkoutResult, setCheckoutResult] = useState<CheckoutResult | null>(null)
  const [isSimulating, setIsSimulating] = useState(false)
  const { entries, append, clear } = useTransactionLog()

  const simulateCheckoutFlow = useCallback(async () => {
    clear()
    setCheckoutResult(null)
    setIsSimulating(true)

    const result = buildDemoCheckoutResult(fundingType)
    const plannedEntries = buildCheckoutLogEntries(result)

    for (const entry of plannedEntries) {
      append(entry.message, entry.level)
      await new Promise((resolve) => setTimeout(resolve, 350))
    }

    setCheckoutResult(result)
    setIsSimulating(false)
  }, [append, clear, fundingType])

  return (
    <main className="flex min-h-screen flex-col items-center justify-center px-6 py-10">
      <div className="w-full max-w-lg rounded-2xl border border-slate-800 bg-slate-900/60 p-8 shadow-xl">
        <p className="text-sm font-medium uppercase tracking-widest text-emerald-400">
          Pasarela Multi-Rail
        </p>
        <h1 className="mt-2 text-3xl font-semibold text-white">Checkout</h1>

        <div className="mt-6 space-y-8">
          <CardForm disabled={isSimulating} onSubmit={setValidatedCard} />
          <RailSelector
            disabled={isSimulating}
            onChange={setFundingType}
            value={fundingType}
          />
          <TransactionViewer entries={entries} result={checkoutResult} />
        </div>

        <button
          className="mt-6 w-full rounded-lg border border-slate-700 px-4 py-2.5 text-sm font-medium text-slate-200 transition hover:border-emerald-500 hover:text-emerald-200 disabled:cursor-not-allowed disabled:opacity-50"
          disabled={isSimulating}
          onClick={() => {
            void simulateCheckoutFlow()
          }}
          type="button"
        >
          {isSimulating ? 'Simulando checkout…' : 'Simular flujo de checkout'}
        </button>

        {validatedCard ? (
          <p className="mt-4 rounded-lg border border-emerald-800/60 bg-emerald-950/40 px-3 py-2 text-sm text-emerald-200">
            Tarjeta validada · titular {validatedCard.cardholder} · terminada en{' '}
            {validatedCard.pan.slice(-4)} · riel {fundingType.replace(/_/g, ' ')}
          </p>
        ) : null}
      </div>
    </main>
  )
}

export default App
