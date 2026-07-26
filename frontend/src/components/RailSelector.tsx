import type { FundingType } from '../schemas/funding'
import { RAIL_OPTIONS } from '../schemas/funding'

export interface RailSelectorProps {
  /** Riel seleccionado actualmente. */
  value: FundingType
  /** Invocado al cambiar de riel. */
  onChange: (rail: FundingType) => void
  /** Deshabilita la selección durante un checkout en curso. */
  disabled?: boolean
}

/**
 * Selector de riel de liquidación (TraditionalBank, BinanceCex, SolanaWallet).
 * Emite valores `snake_case` listos para `POST /api/v1/checkout`.
 */
export function RailSelector({
  value,
  onChange,
  disabled = false,
}: RailSelectorProps) {
  return (
    <fieldset
      aria-label="Riel de liquidación"
      className="space-y-3"
      disabled={disabled}
    >
      <legend className="text-lg font-medium text-white">Riel de liquidación</legend>
      <div className="grid gap-3" role="radiogroup" aria-label="Opciones de riel">
        {RAIL_OPTIONS.map((option) => {
          const selected = value === option.value
          const inputId = `rail-${option.value}`

          return (
            <label
              className={`flex cursor-pointer gap-3 rounded-xl border px-4 py-3 transition ${
                selected
                  ? 'border-emerald-500 bg-emerald-950/40 ring-1 ring-emerald-500/50'
                  : 'border-slate-700 bg-slate-950/40 hover:border-slate-500'
              } ${disabled ? 'cursor-not-allowed opacity-50' : ''}`}
              htmlFor={inputId}
              key={option.value}
            >
              <input
                checked={selected}
                className="mt-1 h-4 w-4 accent-emerald-500"
                disabled={disabled}
                id={inputId}
                name="funding_type"
                onChange={() => onChange(option.value)}
                type="radio"
                value={option.value}
              />
              <span className="flex flex-col">
                <span className="font-medium text-white">{option.label}</span>
                <span className="text-sm text-slate-400">{option.description}</span>
              </span>
            </label>
          )
        })}
      </div>
    </fieldset>
  )
}
