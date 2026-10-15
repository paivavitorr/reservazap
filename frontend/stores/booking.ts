import { defineStore } from 'pinia'
import type {
  AvailabilityResponse,
  BookingIntentPayload,
  BookingIntentResponse,
  BookingStatusResponse,
  CatalogResponse,
  Professional,
  Service,
  Slot,
} from '~/types/booking'

type BookingStep = 'catalog' | 'selecting' | 'hold' | 'confirmed' | 'expired'
type ApiClient = ReturnType<typeof $fetch.create>

/** Fora do state do Pinia: não serializa no payload SSR. */
let api: ApiClient | null = null
/** Guarda contra respostas de disponibilidade fora de ordem. */
let availabilityRequest = 0

interface BookingState {
  step: BookingStep
  catalog: CatalogResponse | null
  availability: AvailabilityResponse | null
  selectedService: Service | null
  selectedProfessional: Professional | null
  selectedDay: string | null
  selectedSlot: Slot | null
  clientName: string
  clientPhone: string
  clientEmail: string
  hold: BookingIntentResponse | null
  error: string | null
  loading: boolean
}

export const useBookingStore = defineStore('booking', {
  state: (): BookingState => ({
    step: 'catalog',
    catalog: null,
    availability: null,
    selectedService: null,
    selectedProfessional: null,
    selectedDay: null,
    selectedSlot: null,
    clientName: '',
    clientPhone: '',
    clientEmail: '',
    hold: null,
    error: null,
    loading: false,
  }),

  getters: {
    signalAmountFormatted: (state) => {
      if (!state.hold) return ''
      return formatCents(state.hold.signal.amount_in_cents)
    },

    holdRemainingSeconds: (state) => {
      if (!state.hold) return 0
      const expires = new Date(state.hold.hold_expires_at).getTime()
      return Math.max(0, Math.floor((expires - Date.now()) / 1000))
    },

    selectedDayLabel: (state) => {
      if (!state.selectedSlot || !state.selectedDay) return ''
      return `${state.selectedDay} às ${state.selectedSlot.label}`
    },

    canSubmit: (state) =>
      !!state.selectedService &&
      !!state.selectedProfessional &&
      !!state.selectedSlot &&
      state.clientName.trim().length >= 2 &&
      state.clientPhone.replace(/\D/g, '').length >= 10,
  },

  actions: {
    /** Chamar no `setup()` da página — fixa a baseURL para uso fora do contexto. */
    init(client: ApiClient) {
      api = client
    },

    async fetchCatalog(slug: string) {
      if (!api) return
      this.loading = true
      this.error = null
      try {
        this.catalog = await api!<CatalogResponse>(`/public/${slug}/catalog`)
        this.step = 'catalog'
      } catch (e: unknown) {
        this.error = mensagemDeErro(e, 'Estabelecimento não encontrado')
      } finally {
        this.loading = false
      }
    },

    async fetchAvailability(slug: string) {
      if (!api || !this.selectedService || !this.selectedProfessional) return
      const pedido = ++availabilityRequest
      this.loading = true
      try {
        const data = await api!<AvailabilityResponse>(`/public/${slug}/availability`, {
          query: {
            service_id: this.selectedService.id,
            professional_id: this.selectedProfessional.id,
            days: 7,
          },
        })
        if (pedido !== availabilityRequest) return // resposta obsoleta
        this.availability = data
        if (!this.selectedDay) {
          const primeiroDia = data.days.find((d) => d.slots.length > 0)
          this.selectedDay = primeiroDia?.date ?? data.days[0]?.date ?? null
        }
      } catch (e: unknown) {
        if (pedido !== availabilityRequest) return
        this.error = mensagemDeErro(e, 'Erro ao carregar horários')
      } finally {
        if (pedido === availabilityRequest) this.loading = false
      }
    },

    async submitBooking(slug: string) {
      if (!api || !this.canSubmit || !this.selectedService || !this.selectedProfessional || !this.selectedSlot) return
      this.loading = true
      this.error = null
      try {
        const payload: BookingIntentPayload = {
          service_id: this.selectedService.id,
          professional_id: this.selectedProfessional.id,
          starts_at: this.selectedSlot.starts_at,
          client_name: this.clientName.trim(),
          client_phone: this.clientPhone,
          client_email: this.clientEmail.trim() || undefined,
        }
        this.hold = await api!<BookingIntentResponse>(`/public/${slug}/bookings/intent`, {
          method: 'POST',
          body: payload,
        })
        this.step = 'hold'
      } catch (e: unknown) {
        this.error = mensagemDeErro(e, 'Erro ao criar agendamento')
      } finally {
        this.loading = false
      }
    },

    async checkStatus(slug: string) {
      if (!api || !this.hold) return
      try {
        const data = await api!<BookingStatusResponse>(
          `/public/${slug}/bookings/${this.hold.appointment_id}/status`
        )
        if (data.status === 'confirmed') {
          this.step = 'confirmed'
        } else if (data.status === 'expired' || data.status === 'cancelled') {
          this.step = 'expired'
        }
      } catch {
        // polling tolera falhas transitórias
      }
    },

    selectService(service: Service) {
      this.selectedService = service
      this.selectedProfessional = null
      this.selectedSlot = null
      this.availability = null
      this.selectedDay = null
      this.step = 'selecting'
    },

    selectProfessional(professional: Professional) {
      this.selectedProfessional = professional
      this.selectedSlot = null
      this.availability = null
      this.selectedDay = null
    },

    selectSlot(slot: Slot) {
      this.selectedSlot = slot
    },

    reset() {
      this.$reset()
    },
  },
})

function mensagemDeErro(e: unknown, fallback: string): string {
  const err = e as { data?: { description?: string; message?: string }; message?: string }
  return err?.data?.description || err?.data?.message || err?.message || fallback
}
