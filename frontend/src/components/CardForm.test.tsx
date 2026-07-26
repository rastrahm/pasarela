import { describe, expect, it, vi } from 'vitest'

import { DEMO_CARD } from '../schemas/card'
import { CardForm } from './CardForm'
import { renderUi, screen, userEvent } from '../test/test-utils'

describe('CardForm', () => {
  it('renderiza los campos de tarjeta', () => {
    renderUi(<CardForm onSubmit={vi.fn()} />)

    expect(screen.getByLabelText(/número de tarjeta/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/mes/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/año/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/cvv/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/titular/i)).toBeInTheDocument()
  })

  it('muestra errores si se envía vacío', async () => {
    const user = userEvent.setup()
    renderUi(<CardForm onSubmit={vi.fn()} />)

    await user.click(screen.getByRole('button', { name: /validar tarjeta/i }))

    expect(screen.getByText(/revisá los datos/i)).toBeInTheDocument()
  })

  it('rechaza un PAN inválido', async () => {
    const user = userEvent.setup()
    const onSubmit = vi.fn()
    renderUi(<CardForm onSubmit={onSubmit} />)

    await user.click(screen.getByRole('button', { name: /usar datos de prueba/i }))
    await user.clear(screen.getByLabelText(/número de tarjeta/i))
    await user.type(screen.getByLabelText(/número de tarjeta/i), '4111111111111112')
    await user.click(screen.getByRole('button', { name: /validar tarjeta/i }))

    expect(screen.getByText(/luhn/i)).toBeInTheDocument()
    expect(onSubmit).not.toHaveBeenCalled()
  })

  it('envía datos válidos al callback onSubmit', async () => {
    const user = userEvent.setup()
    const onSubmit = vi.fn()
    renderUi(<CardForm onSubmit={onSubmit} />)

    await user.click(screen.getByRole('button', { name: /usar datos de prueba/i }))
    await user.click(screen.getByRole('button', { name: /validar tarjeta/i }))

    expect(onSubmit).toHaveBeenCalledOnce()
    expect(onSubmit).toHaveBeenCalledWith(DEMO_CARD)
  })

  it('rellena datos ficticios con el botón de prueba', async () => {
    const user = userEvent.setup()
    renderUi(<CardForm onSubmit={vi.fn()} />)

    await user.click(screen.getByRole('button', { name: /usar datos de prueba/i }))

    expect(screen.getByLabelText(/número de tarjeta/i)).toHaveValue(DEMO_CARD.pan)
    expect(screen.getByLabelText(/titular/i)).toHaveValue(DEMO_CARD.cardholder)
  })

  it('respeta la prop disabled', () => {
    renderUi(<CardForm disabled onSubmit={vi.fn()} />)

    expect(screen.getByLabelText(/número de tarjeta/i)).toBeDisabled()
    expect(screen.getByRole('button', { name: /validar tarjeta/i })).toBeDisabled()
  })
})
