import { defineStore } from 'pinia'
import type {
  CatalogResponse,
  BookingIntentPayload,
  BookingIntentResponse,
  BookingStatusResponse,
  Service,
  Professional,
} from '~/types/booking'

type BookingStep = 'catalog' | 'selecting' | 'hold' | 'confirmed' | 'expired'

interface BookingState {
  step: BookingStep
  catalog: CatalogResponse | null
  selectedService: Service | null
  selectedProfessional: Professional | null
  selectedTime: string | null
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
    selectedService: null,
    selectedProfessional: null,
    selectedTime: null,
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
      return (state.hold.signal.amount_in_cents / 100).toLocaleString('pt-BR', {
        style: 'currency',
        currency: 'BRL',
      })
    },

    holdRemainingSeconds: (state) => {
      if (!state.hold) return 0
      const expires = new Date(state.hold.hold_expires_at).getTime()
      const now = Date.now()
      return Math.max(0, Math.floor((expires - now) / 1000))
    },

    canSubmit: (state) =>
      !!state.selectedService &&
      !!state.selectedProfessional &&
      !!state.selectedTime &&
      state.clientName.trim().length > 0 &&
      state.clientPhone.trim().length > 0,
  },

  actions: {
    async fetchCatalog(slug: string) {
      this.loading = true
      this.error = null
      try {
        const config = useRuntimeConfig()
        const data = await $fetch<CatalogResponse>(
          `${config.public.apiBase}/public/${slug}/catalog`
        )
        this.catalog = data
        this.step = 'catalog'
      } catch (e: any) {
        this.error = e.data?.message || 'Erro ao carregar catálogo'
      } finally {
        this.loading = false
      }
    },

    async submitBooking(slug: string) {
      if (!this.canSubmit || !this.selectedService || !this.selectedProfessional || !this.selectedTime) return

      this.loading = true
      this.error = null
      try {
        const config = useRuntimeConfig()
        const payload: BookingIntentPayload = {
          service_id: this.selectedService.id,
          professional_id: this.selectedProfessional.id,
          starts_at: this.selectedTime,
          client_name: this.clientName,
          client_phone: this.clientPhone,
          client_email: this.clientEmail || undefined,
        }
        const data = await $fetch<BookingIntentResponse>(
          `${config.public.apiBase}/public/${slug}/bookings/intent`,
          { method: 'POST', body: payload }
        )
        this.hold = data
        this.step = 'hold'
      } catch (e: any) {
        this.error = e.data?.message || 'Erro ao criar agendamento'
      } finally {
        this.loading = false
      }
    },

    async checkStatus(slug: string) {
      if (!this.hold) return
      try {
        const config = useRuntimeConfig()
        const data = await $fetch<BookingStatusResponse>(
          `${config.public.apiBase}/public/${slug}/bookings/${this.hold.appointment_id}/status`
        )
        if (data.status === 'confirmed') {
          this.step = 'confirmed'
        } else if (data.status === 'expired') {
          this.step = 'expired'
        }
      } catch {
        // silencioso — polling tolera falhas
      }
    },

    selectService(service: Service) {
      this.selectedService = service
      this.step = 'selecting'
    },

    reset() {
      this.$reset()
    },
  },
})