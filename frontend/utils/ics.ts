/** Gera o conteúdo de um evento .ics (UTC) para o agendamento. */
export function gerarIcs(params: {
  uid: string
  titulo: string
  descricao: string
  local: string
  inicioUtc: string
  duracaoMinutos: number
}): string {
  const inicio = new Date(params.inicioUtc)
  const fim = new Date(inicio.getTime() + params.duracaoMinutos * 60_000)
  const fmt = (d: Date) => d.toISOString().replace(/[-:]/g, '').replace(/\.\d{3}/, '')

  return [
    'BEGIN:VCALENDAR',
    'VERSION:2.0',
    'PRODID:-//ReservaZap//PT-BR',
    'BEGIN:VEVENT',
    `UID:reservazap-${params.uid}@reservazap`,
    `DTSTAMP:${fmt(new Date())}`,
    `DTSTART:${fmt(inicio)}`,
    `DTEND:${fmt(fim)}`,
    `SUMMARY:${escapeIcs(params.titulo)}`,
    `DESCRIPTION:${escapeIcs(params.descricao)}`,
    `LOCATION:${escapeIcs(params.local)}`,
    'END:VEVENT',
    'END:VCALENDAR',
  ].join('\r\n')
}

export function baixarArquivo(nome: string, conteudo: string, tipo = 'text/calendar') {
  const blob = new Blob([conteudo], { type: `${tipo};charset=utf-8` })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = nome
  a.click()
  URL.revokeObjectURL(url)
}

function escapeIcs(valor: string): string {
  return valor.replace(/\\/g, '\\\\').replace(/;/g, '\\;').replace(/,/g, '\\,').replace(/\n/g, '\\n')
}
