import { render, type RenderOptions } from '@testing-library/react'
import type { ReactElement } from 'react'

/**
 * Punto único de render para tests de componentes (RTL).
 *
 * @param ui - Elemento React a montar.
 * @param options - Opciones de `@testing-library/react` `render`.
 * @returns Resultado de `render` (queries, container, unmount).
 */
export function renderUi(ui: ReactElement, options?: RenderOptions) {
  return render(ui, options)
}

export * from '@testing-library/react'
export { default as userEvent } from '@testing-library/user-event'
