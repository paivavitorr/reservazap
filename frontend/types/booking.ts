export interface Tenant {
  id: string
  name: string
  slug: string
}

export interface Service {
  id: string
  name: string
  description: string | null
  duration_minutes: number
  price_in_cents: number
  signal_percent: number
}

export interface Professional {
  id: string
  name: string
}

export interface CatalogResponse {
  tenant: Tenant
  services: Service[]
  professionals: Professional[]
}

export interface BookingIntentPayload {
  service_id: string
  professional_id: string
  starts_at: string
  client_name: string
  client_phone: string
  client_email?: string
}

export interface BookingIntentResponse {
  appointment_id: string
  status: string
  hold_expires_at: string
  signal: {
    amount_in_cents: number
    pix: {
      txid: string
      copia_e_cola: string
      qr_code_base64: string | null
      expiracao_segundos: number
    }
  }
}

export interface BookingStatusResponse {
  status: string
  hold_expires_at: string | null
  signal_paid_at: string | null
}