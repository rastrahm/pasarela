import { z } from 'zod'

/** Tarjeta de prueba Visa válida (Luhn OK) — alineada con tests del Gateway/Oracle. */
export const DEMO_CARD: CardPayload = {
  pan: '4111111111111111',
  expiry_month: '12',
  expiry_year: '2030',
  cvv: '123',
  cardholder: 'Demo User',
}

export type CardBrand = 'visa' | 'mastercard' | 'amex' | 'unknown'

export type CardPayload = {
  pan: string
  expiry_month: string
  expiry_year: string
  cvv: string
  cardholder: string
}

/** Normaliza PAN eliminando espacios y guiones. */
export function normalizePan(value: string): string {
  return value.replace(/[\s-]/g, '')
}

/** Valida el PAN con el algoritmo de Luhn. */
export function luhnValid(digits: string): boolean {
  if (!/^\d+$/.test(digits)) {
    return false
  }

  let sum = 0
  let alternate = false

  for (let index = digits.length - 1; index >= 0; index -= 1) {
    let digit = Number.parseInt(digits[index] ?? '', 10)
    if (Number.isNaN(digit)) {
      return false
    }

    if (alternate) {
      digit *= 2
      if (digit > 9) {
        digit -= 9
      }
    }

    sum += digit
    alternate = !alternate
  }

  return sum % 10 === 0
}

/** Detecta marca a partir de dígitos del PAN (misma heurística que el Oracle). */
export function detectCardBrand(digits: string): CardBrand {
  if (digits.startsWith('4')) {
    return 'visa'
  }

  if (digits.length >= 2) {
    const prefix2 = Number.parseInt(digits.slice(0, 2), 10)
    if (prefix2 >= 51 && prefix2 <= 55) {
      return 'mastercard'
    }
    if (digits.startsWith('34') || digits.startsWith('37')) {
      return 'amex'
    }
  }

  return 'unknown'
}

const monthSchema = z
  .string()
  .trim()
  .regex(/^(0?[1-9]|1[0-2])$/, 'Mes inválido (01–12)')

const yearSchema = z
  .string()
  .trim()
  .regex(/^(\d{2}|\d{4})$/, 'Año inválido')
  .transform((value) => (value.length === 2 ? `20${value}` : value))
  .refine((value) => /^\d{4}$/.test(value), 'Año inválido')

const cardholderSchema = z
  .string()
  .trim()
  .min(2, 'Nombre del titular requerido')
  .max(80, 'Nombre demasiado largo')
  .regex(/^[A-Za-zÀ-ÿ\s'.-]+$/, 'Solo letras y espacios')

/** Schema Zod alineado con `CheckoutCardPayload` del API Gateway. */
export const cardPayloadSchema = z
  .object({
    pan: z
      .string()
      .trim()
      .transform(normalizePan)
      .refine((value) => /^\d{13,19}$/.test(value), 'Número de tarjeta inválido')
      .refine(luhnValid, 'Número de tarjeta no pasa Luhn')
      .refine(
        (value) => detectCardBrand(value) !== 'unknown',
        'Marca de tarjeta no soportada (Visa, Mastercard o Amex)',
      ),
    expiry_month: monthSchema,
    expiry_year: yearSchema,
    cvv: z.string().trim(),
    cardholder: cardholderSchema,
  })
  .superRefine((value, context) => {
    const brand = detectCardBrand(value.pan)
    const cvvPattern = brand === 'amex' ? /^\d{4}$/ : /^\d{3}$/

    if (!cvvPattern.test(value.cvv)) {
      context.addIssue({
        code: 'custom',
        message:
          brand === 'amex'
            ? 'CVV debe tener 4 dígitos (Amex)'
            : 'CVV debe tener 3 dígitos',
        path: ['cvv'],
      })
    }

    const month = Number.parseInt(value.expiry_month.padStart(2, '0'), 10)
    const year = Number.parseInt(value.expiry_year, 10)
    const expiresAt = new Date(year, month, 0, 23, 59, 59, 999)
    if (expiresAt.getTime() < Date.now()) {
      context.addIssue({
        code: 'custom',
        message: 'Tarjeta vencida',
        path: ['expiry_year'],
      })
    }
  })
  .transform((value) => ({
    ...value,
    expiry_month: value.expiry_month.padStart(2, '0'),
  }))

export type CardFormValues = z.input<typeof cardPayloadSchema>
export type ValidatedCardPayload = z.output<typeof cardPayloadSchema>

/** Valida datos de tarjeta; devuelve errores por campo para el formulario. */
export function validateCardPayload(
  values: CardFormValues,
): { success: true; data: ValidatedCardPayload } | { success: false; errors: Partial<Record<keyof CardPayload, string>> } {
  const result = cardPayloadSchema.safeParse(values)

  if (result.success) {
    return { success: true, data: result.data }
  }

  const errors: Partial<Record<keyof CardPayload, string>> = {}
  for (const issue of result.error.issues) {
    const field = issue.path[0]
    if (typeof field === 'string' && !(field in errors)) {
      errors[field as keyof CardPayload] = issue.message
    }
  }

  return { success: false, errors }
}
