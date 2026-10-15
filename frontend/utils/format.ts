export function formatCents(cents: number): string {
  return (cents / 100)
    .toLocaleString('pt-BR', { style: 'currency', currency: 'BRL' })
    .replace(/\u00A0/g, ' ')
}

export function formatMinutes(min: number): string {
  return `${min} min`
}

/** "2026-10-16" → "16/10" */
export function formatShortDate(isoDate: string): string {
  const [, mes, dia] = isoDate.split('-')
  return `${dia}/${mes}`
}

/** "2026-10-16" → "quinta-feira, 16/10" */
export function formatLongDate(isoDate: string, weekday: string): string {
  return `${weekday}, ${formatShortDate(isoDate)}`
}

/**
 * Máscara de telefone BR. Aceita DDI: com 12–13 dígitos iniciando em 55,
 * formata como +55 (11) 98765-4321. Números com mais de 11 dígitos sem DDI
 * são mantidos sem máscara para não corromper o valor.
 */
export function maskPhone(value: string): string {
  const digits = value.replace(/\D/g, '')
  if (digits.length > 11 && digits.startsWith('55')) {
    return `+55 ${maskPhone(digits.slice(2))}`
  }
  if (digits.length > 11) return digits
  if (digits.length <= 2) return digits
  if (digits.length <= 6) return `(${digits.slice(0, 2)}) ${digits.slice(2)}`
  if (digits.length <= 10) return `(${digits.slice(0, 2)}) ${digits.slice(2, 6)}-${digits.slice(6)}`
  return `(${digits.slice(0, 2)}) ${digits.slice(2, 7)}-${digits.slice(7)}`
}

/** mm:ss a partir de segundos restantes */
export function formatCountdown(totalSeconds: number): string {
  const safe = Math.max(0, totalSeconds)
  const min = Math.floor(safe / 60)
  const seg = safe % 60
  return `${String(min).padStart(2, '0')}:${String(seg).padStart(2, '0')}`
}
