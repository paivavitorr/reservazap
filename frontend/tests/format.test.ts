import { describe, expect, it } from 'vitest'
import {
  formatCountdown,
  formatCents,
  formatMinutes,
  formatShortDate,
  maskPhone,
} from '../utils/format'

describe('formatCents', () => {
  it('formata centavos em BRL', () => {
    expect(formatCents(1350)).toBe('R$ 13,50')
    expect(formatCents(0)).toBe('R$ 0,00')
    expect(formatCents(650000)).toBe('R$ 6.500,00')
  })
})

describe('formatMinutes', () => {
  it('acrescenta sufixo min', () => {
    expect(formatMinutes(30)).toBe('30 min')
    expect(formatMinutes(50)).toBe('50 min')
  })
})

describe('formatShortDate', () => {
  it('converte YYYY-MM-DD em DD/MM', () => {
    expect(formatShortDate('2026-10-16')).toBe('16/10')
    expect(formatShortDate('2026-01-05')).toBe('05/01')
  })
})

describe('maskPhone', () => {
  it('máscara parcial e completa', () => {
    expect(maskPhone('11')).toBe('11')
    expect(maskPhone('119876')).toBe('(11) 9876')
    expect(maskPhone('1198765432')).toBe('(11) 9876-5432')
    expect(maskPhone('11987654321')).toBe('(11) 98765-4321')
  })

  it('preserva DDI 55 sem corromper', () => {
    expect(maskPhone('5511987654321')).toBe('+55 (11) 98765-4321')
    expect(maskPhone('551198765432')).toBe('+55 (11) 9876-5432')
    expect(maskPhone('55119876543219')).toBe('+55 119876543219')
  })

  it('remove caracteres não numéricos', () => {
    expect(maskPhone('(11) 98765-4321')).toBe('(11) 98765-4321')
    expect(maskPhone('abc11def987654321')).toBe('(11) 98765-4321')
  })
})

describe('formatCountdown', () => {
  it('formata mm:ss', () => {
    expect(formatCountdown(0)).toBe('00:00')
    expect(formatCountdown(9)).toBe('00:09')
    expect(formatCountdown(600)).toBe('10:00')
    expect(formatCountdown(-5)).toBe('00:00')
  })
})
