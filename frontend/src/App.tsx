import { useState } from 'react'

import { CardForm } from './components/CardForm'
import type { CardPayload } from './schemas/card'

/**
 * Shell inicial de la pasarela. Los componentes de checkout se añaden en pasos 5.4–5.6.
 */
function App() {
  const [validatedCard, setValidatedCard] = useState<CardPayload | null>(null)

  return (
    <main className="flex min-h-screen flex-col items-center justify-center px-6 py-10">
      <div className="w-full max-w-lg rounded-2xl border border-slate-800 bg-slate-900/60 p-8 shadow-xl">
        <p className="text-sm font-medium uppercase tracking-widest text-emerald-400">
          Pasarela Multi-Rail
        </p>
        <h1 className="mt-2 text-3xl font-semibold text-white">Checkout</h1>

        <div className="mt-6">
          <CardForm onSubmit={setValidatedCard} />
        </div>

        {validatedCard ? (
          <p className="mt-4 rounded-lg border border-emerald-800/60 bg-emerald-950/40 px-3 py-2 text-sm text-emerald-200">
            Tarjeta validada · titular {validatedCard.cardholder} · terminada en{' '}
            {validatedCard.pan.slice(-4)}
          </p>
        ) : null}
      </div>
    </main>
  )
}

export default App
