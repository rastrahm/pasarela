import { describe, expect, it } from 'vitest'

import App from './App'
import { renderUi, screen } from './test/test-utils'

describe('App', () => {
  it('muestra el encabezado del checkout', () => {
    renderUi(<App />)

    expect(screen.getByRole('heading', { name: /checkout/i })).toBeInTheDocument()
  })

  it('muestra la marca Pasarela Multi-Rail', () => {
    renderUi(<App />)

    expect(screen.getByText(/pasarela multi-rail/i)).toBeInTheDocument()
  })
})
