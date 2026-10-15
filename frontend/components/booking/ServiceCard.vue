<script setup lang="ts">
import type { Service } from '~/types/booking'

const props = defineProps<{ service: Service; selected: boolean }>()
const emit = defineEmits<{ select: [] }>()
</script>

<template>
  <button
    type="button"
    class="w-full rounded-lg border-2 bg-white px-4 py-3.5 text-left transition-colors"
    :class="props.selected ? 'border-brand bg-brand-soft/40' : 'border-ink/10 hover:border-ink/25'"
    :aria-pressed="props.selected"
    @click="emit('select')"
  >
    <div class="flex items-start justify-between gap-3">
      <div class="min-w-0">
        <p class="font-medium leading-snug">{{ props.service.name }}</p>
        <p v-if="props.service.description" class="mt-0.5 text-sm leading-snug text-ink/60">
          {{ props.service.description }}
        </p>
        <p class="mt-1 text-xs text-ink/50">
          {{ formatMinutes(props.service.duration_minutes) }} · sinal de {{ props.service.signal_percent }}%
        </p>
      </div>
      <p class="shrink-0 font-display text-base font-semibold">
        {{ formatCents(props.service.price_in_cents) }}
      </p>
    </div>
  </button>
</template>
