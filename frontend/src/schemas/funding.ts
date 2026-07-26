import { z } from 'zod'

/** Valores `snake_case` alineados con `FundingType` del dominio y API Gateway. */
export const fundingTypeSchema = z.enum([
  'traditional_bank',
  'binance_cex',
  'solana_wallet',
])

export type FundingType = z.infer<typeof fundingTypeSchema>

export const DEFAULT_FUNDING_TYPE: FundingType = 'traditional_bank'

export interface RailOption {
  value: FundingType
  label: string
  description: string
}

/** Opciones de riel expuestas en el checkout. */
export const RAIL_OPTIONS: RailOption[] = [
  {
    value: 'traditional_bank',
    label: 'Banco tradicional',
    description: 'Liquidación fiat simulada (ACH / SEPA)',
  },
  {
    value: 'binance_cex',
    label: 'Binance CEX',
    description: 'Saldo custodial en exchange simulado',
  },
  {
    value: 'solana_wallet',
    label: 'Solana Wallet',
    description: 'Liquidación on-chain (devnet)',
  },
]

/** Comprueba que un valor es un `FundingType` válido. */
export function isFundingType(value: string): value is FundingType {
  return fundingTypeSchema.safeParse(value).success
}
