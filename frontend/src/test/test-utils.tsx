import { render, type RenderOptions } from '@testing-library/react'
import type { ReactElement } from 'react'

/**
 * Punto único de render para tests de componentes.
 * En pasos posteriores se pueden añadir providers (Context, router, etc.).
 */
export function renderUi(ui: ReactElement, options?: RenderOptions) {
  return render(ui, options)
}

export * from '@testing-library/react'
export { default as userEvent } from '@testing-library/user-event'
