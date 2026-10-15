<script setup lang="ts">
import type { Professional, Service, Slot } from '~/types/booking'

const props = defineProps<{
  service: Service
  professional: Professional
  day: string
  slot: Slot
  clientName: string
  clientPhone: string
}>()

// mesma truncagem inteira do backend (price * percent / 100)
const sinal = computed(() => Math.floor((props.service.price_in_cents * props.service.signal_percent) / 100))
const restante = computed(() => props.service.price_in_cents - sinal.value)
</script>

<template>
  <div class="rounded-lg border border-ink/10 bg-white px-4 py-4">
    <dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-sm">
      <dt class="text-ink/55">Serviço</dt>
      <dd class="text-right font-medium">{{ props.service.name }}</dd>

      <dt class="text-ink/55">Profissional</dt>
      <dd class="text-right font-medium">{{ props.professional.name }}</dd>

      <dt class="text-ink/55">Data e hora</dt>
      <dd class="text-right font-medium">
        {{ formatShortDate(props.day) }} · {{ props.slot.label }}
      </dd>

      <dt class="text-ink/55">Cliente</dt>
      <dd class="text-right font-medium">{{ props.clientName }}</dd>

      <dt class="text-ink/55">Valor total</dt>
      <dd class="text-right font-medium">{{ formatCents(props.service.price_in_cents) }}</dd>
    </dl>
    <div class="mt-3 border-t border-dashed border-ink/20 pt-3">
      <div class="flex items-baseline justify-between">
        <p class="text-sm text-ink/55">Sinal agora ({{ props.service.signal_percent }}%)</p>
        <p class="font-display text-lg font-semibold text-brand">{{ formatCents(sinal) }}</p>
      </div>
      <p class="mt-0.5 text-right text-xs text-ink/50">
        Restante de {{ formatCents(restante) }} pago no local
      </p>
    </div>
  </div>
</template>
