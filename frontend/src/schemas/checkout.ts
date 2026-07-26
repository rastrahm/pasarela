import { z } from 'zod'

import { cardPayloadSchema } from './card'
import { fundingTypeSchema } from './funding'

/** Solicitud de checkout alineada con `CheckoutRequest` del Gateway. */
export const checkoutRequestSchema = z.object({
  amount: z.number().positive('El monto debe ser mayor a 0'),
  currency: z
    .string()
    .trim()
    .length(3, 'La moneda debe tener 3 letras (ISO 4217)')
    .transform((value) => value.toUpperCase()),
  funding_type: fundingTypeSchema.optional(),
  card: cardPayloadSchema,
})

export type CheckoutRequest = z.output<typeof checkoutRequestSchema>
export type CheckoutRequestInput = z.input<typeof checkoutRequestSchema>

/** Valida el payload antes de enviarlo al Gateway. */
export function parseCheckoutRequest(input: CheckoutRequestInput): CheckoutRequest {
  return checkoutRequestSchema.parse(input)
}
