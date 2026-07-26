import { describe, expect, it, vi } from 'vitest'

import { DEFAULT_FUNDING_TYPE } from '../schemas/funding'
import { RailSelector } from './RailSelector'
import { renderUi, screen, userEvent } from '../test/test-utils'

describe('RailSelector', () => {
  it('renderiza los tres rieles disponibles', () => {
    renderUi(
      <RailSelector onChange={vi.fn()} value={DEFAULT_FUNDING_TYPE} />,
    )

    expect(screen.getByRole('radio', { name: /banco tradicional/i })).toBeInTheDocument()
    expect(screen.getByRole('radio', { name: /binance cex/i })).toBeInTheDocument()
    expect(screen.getByRole('radio', { name: /solana wallet/i })).toBeInTheDocument()
  })

  it('marca el riel seleccionado', () => {
    renderUi(
      <RailSelector onChange={vi.fn()} value="binance_cex" />,
    )

    expect(screen.getByRole('radio', { name: /binance cex/i })).toBeChecked()
    expect(screen.getByRole('radio', { name: /banco tradicional/i })).not.toBeChecked()
  })

  it('notifica el cambio de riel', async () => {
    const user = userEvent.setup()
    const onChange = vi.fn()
    renderUi(
      <RailSelector onChange={onChange} value={DEFAULT_FUNDING_TYPE} />,
    )

    await user.click(screen.getByRole('radio', { name: /solana wallet/i }))

    expect(onChange).toHaveBeenCalledOnce()
    expect(onChange).toHaveBeenCalledWith('solana_wallet')
  })

  it('respeta la prop disabled', () => {
    renderUi(
      <RailSelector disabled onChange={vi.fn()} value={DEFAULT_FUNDING_TYPE} />,
    )

    expect(screen.getByRole('radio', { name: /banco tradicional/i })).toBeDisabled()
    expect(screen.getByRole('radio', { name: /binance cex/i })).toBeDisabled()
    expect(screen.getByRole('radio', { name: /solana wallet/i })).toBeDisabled()
  })

  it('expone un radiogroup accesible', () => {
    renderUi(
      <RailSelector onChange={vi.fn()} value={DEFAULT_FUNDING_TYPE} />,
    )

    expect(screen.getByRole('radiogroup', { name: /opciones de riel/i })).toBeInTheDocument()
  })
})
