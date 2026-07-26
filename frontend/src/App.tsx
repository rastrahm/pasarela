import { useState } from 'react'

import { CardForm } from './components/CardForm'
import { RailSelector } from './components/RailSelector'
import type { CardPayload } from './schemas/card'
import { DEFAULT_FUNDING_TYPE, type FundingType } from './schemas/funding'

/**
 * Shell inicial de la pasarela. CheckoutPage completo en paso 5.6.
 */
function App() {
  const [validatedCard, setValidatedCard] = useState<CardPayload | null>(null)
  const [fundingType, setFundingType] = useState<FundingType>(DEFAULT_FUNDING_TYPE)

  return (
    <main className="flex min-h-screen flex-col items-center justify-center px-6 py-10">
      <div className="w-full max-w-lg rounded-2xl border border-slate-800 bg-slate-900/60 p-8 shadow-xl">
        <p className="text-sm font-medium uppercase tracking-widest text-emerald-400">
          Pasarela Multi-Rail
        </p>
        <h1 className="mt-2 text-3xl font-semibold text-white">Checkout</h1>

        <div className="mt-6 space-y-8">
          <CardForm onSubmit={setValidatedCard} />
          <RailSelector onChange={setFundingType} value={fundingType} />
        </div>

        {validatedCard ? (
          <p className="mt-4 rounded-lg border border-emerald-800/60 bg-emerald-950/40 px-3 py-2 text-sm text-emerald-200">
            Tarjeta validada · titular {validatedCard.cardholder} · terminada en{' '}
            {validatedCard.pan.slice(-4)} · riel{' '}
            {fundingType.replace(/_/g, ' ')}
          </p>
        ) : null}
      </div>
    </main>
  )
}

export default App
