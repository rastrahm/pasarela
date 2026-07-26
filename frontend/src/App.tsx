/**
 * Shell inicial de la pasarela. Los componentes de checkout se añaden en pasos 5.3–5.6.
 */
function App() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-center px-6">
      <div className="w-full max-w-lg rounded-2xl border border-slate-800 bg-slate-900/60 p-8 shadow-xl">
        <p className="text-sm font-medium uppercase tracking-widest text-emerald-400">
          Pasarela Multi-Rail
        </p>
        <h1 className="mt-2 text-3xl font-semibold text-white">Checkout</h1>
        <p className="mt-3 text-slate-400">
          Vitest y React Testing Library listos. Próximo paso: componentes de checkout
          (CardForm, RailSelector, TransactionViewer).
        </p>
      </div>
    </main>
  )
}

export default App
