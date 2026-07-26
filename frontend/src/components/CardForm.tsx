import { useCallback, useState, type FormEvent } from 'react'

import {
  DEMO_CARD,
  type CardPayload,
  type CardFormValues,
  validateCardPayload,
} from '../schemas/card'

export type CardFormField = keyof CardPayload

/**
 * Props de {@link CardForm}.
 */
export interface CardFormProps {
  /** Invocado con datos validados listos para el checkout. */
  onSubmit: (card: CardPayload) => void
  /** Deshabilita inputs y envío mientras un checkout está en curso. */
  disabled?: boolean
}

const EMPTY_VALUES: CardFormValues = {
  pan: '',
  expiry_month: '',
  expiry_year: '',
  cvv: '',
  cardholder: '',
}

const FIELD_LABELS: Record<CardFormField, string> = {
  pan: 'Número de tarjeta',
  expiry_month: 'Mes (MM)',
  expiry_year: 'Año (AAAA)',
  cvv: 'CVV',
  cardholder: 'Titular',
}

/**
 * Formulario de tarjeta ficticia con validación Zod local (UC-01).
 *
 * @param props - {@link CardFormProps}
 * @returns Formulario accesible con campos PAN, vencimiento, CVV y titular.
 */
export function CardForm({ onSubmit, disabled = false }: CardFormProps) {
  const [values, setValues] = useState<CardFormValues>(EMPTY_VALUES)
  const [errors, setErrors] = useState<Partial<Record<CardFormField, string>>>({})
  const [submitted, setSubmitted] = useState(false)

  const updateField = useCallback((field: CardFormField, value: string) => {
    setValues((current) => ({ ...current, [field]: value }))
    setErrors((current) => {
      if (!current[field]) {
        return current
      }
      const next = { ...current }
      delete next[field]
      return next
    })
  }, [])

  const fillDemoData = useCallback(() => {
    setValues(DEMO_CARD)
    setErrors({})
    setSubmitted(false)
  }, [])

  const handleSubmit = useCallback(
    (event: FormEvent<HTMLFormElement>) => {
      event.preventDefault()
      setSubmitted(true)

      const result = validateCardPayload(values)
      if (!result.success) {
        setErrors(result.errors)
        return
      }

      setErrors({})
      onSubmit(result.data)
    },
    [onSubmit, values],
  )

  return (
    <form
      aria-label="Datos de tarjeta"
      className="space-y-4"
      noValidate
      onSubmit={handleSubmit}
    >
      <div className="flex items-center justify-between gap-3">
        <h2 className="text-lg font-medium text-white">Tarjeta</h2>
        <button
          className="rounded-lg border border-slate-700 px-3 py-1.5 text-sm text-emerald-300 transition hover:border-emerald-500 hover:text-emerald-200 disabled:cursor-not-allowed disabled:opacity-50"
          disabled={disabled}
          onClick={fillDemoData}
          type="button"
        >
          Usar datos de prueba
        </button>
      </div>

      {(Object.keys(FIELD_LABELS) as CardFormField[]).map((field) => (
        <div key={field}>
          <label className="mb-1 block text-sm text-slate-300" htmlFor={field}>
            {FIELD_LABELS[field]}
          </label>
          <input
            aria-invalid={Boolean(errors[field])}
            autoComplete={field === 'pan' ? 'cc-number' : field === 'cvv' ? 'cc-csc' : field === 'cardholder' ? 'cc-name' : field === 'expiry_month' ? 'cc-exp-month' : 'cc-exp-year'}
            className="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-white outline-none ring-emerald-500/40 focus:ring-2 disabled:cursor-not-allowed disabled:opacity-50"
            disabled={disabled}
            id={field}
            inputMode={
              field === 'cardholder' ? 'text' : 'numeric'
            }
            name={field}
            onChange={(event) => updateField(field, event.target.value)}
            type={field === 'cvv' ? 'password' : 'text'}
            value={values[field]}
          />
          {errors[field] ? (
            <p className="mt-1 text-sm text-rose-400" role="alert">
              {errors[field]}
            </p>
          ) : null}
        </div>
      ))}

      {submitted && Object.keys(errors).length > 0 ? (
        <p className="text-sm text-rose-300" role="alert">
          Revisá los datos de la tarjeta antes de continuar.
        </p>
      ) : null}

      <button
        className="w-full rounded-lg bg-emerald-500 px-4 py-2.5 font-medium text-slate-950 transition hover:bg-emerald-400 disabled:cursor-not-allowed disabled:opacity-50"
        disabled={disabled}
        type="submit"
      >
        Validar tarjeta
      </button>
    </form>
  )
}
