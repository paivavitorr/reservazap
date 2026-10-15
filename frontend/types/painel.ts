export interface UsuarioPainel {
  id: string
  name: string
  email: string
  tenant_id: string
}

export interface AuthResponse {
  token: string
  user: UsuarioPainel
}

export interface ServicoPainel {
  id: string
  name: string
  description: string | null
  duration_minutes: number
  price_in_cents: number
  signal_percent: number
  is_active: boolean
}

export interface ProfissionalPainel {
  id: string
  name: string
  email: string | null
  phone: string | null
  is_active: boolean
}

export interface FaixaExpediente {
  weekday: number
  start: string
  end: string
}

export interface AgendamentoPainel {
  id: string
  starts_at: string
  ends_at: string
  client_name: string
  client_phone: string
  status: string
  service: string
  professional: string
  signal_amount_in_cents: number
  signal_paid_at: string | null
  cancellation_reason: string | null
}

export interface ResumoPainel {
  hoje: number
  semana: number
  confirmados_hoje: number
  taxa_no_show_30d: number
  sinais_pagos_30d: number
}
