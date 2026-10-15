<script setup lang="ts">
import type { AvailabilityResponse, Slot } from '~/types/booking'

const props = defineProps<{
  availability: AvailabilityResponse
  selectedDay: string | null
  selectedSlot: Slot | null
}>()
const emit = defineEmits<{ selectDay: [date: string]; selectSlot: [slot: Slot] }>()

const diaAtual = computed(() =>
  props.availability.days.find((d) => d.date === props.selectedDay) ?? null
)
</script>

<template>
  <div>
    <!-- Dias -->
    <div class="-mx-1 flex gap-2 overflow-x-auto px-1 pb-2" role="tablist" aria-label="Escolha o dia">
      <button
        v-for="day in props.availability.days"
        :key="day.date"
        type="button"
        role="tab"
        class="shrink-0 rounded-lg border-2 px-3 py-2 text-center transition-colors"
        :class="[
          day.date === props.selectedDay
            ? 'border-brand bg-brand text-white'
            : day.slots.length > 0
              ? 'border-ink/10 bg-white hover:border-ink/25'
              : 'border-ink/5 bg-white opacity-40',
        ]"
        :aria-selected="day.date === props.selectedDay"
        :disabled="day.slots.length === 0"
        @click="emit('selectDay', day.date)"
      >
        <p class="text-xs leading-tight opacity-80">{{ day.weekday.split('-')[0] }}</p>
        <p class="font-display text-sm font-semibold leading-tight">{{ formatShortDate(day.date) }}</p>
      </button>
    </div>

    <!-- Horários -->
    <div v-if="diaAtual" class="mt-3">
      <p v-if="diaAtual.slots.length === 0" class="rounded-lg bg-white px-4 py-6 text-center text-sm text-ink/60">
        Sem horários livres neste dia. Escolha outro dia.
      </p>
      <div v-else class="grid grid-cols-3 gap-2 sm:grid-cols-4">
        <button
          v-for="slot in diaAtual.slots"
          :key="slot.starts_at"
          type="button"
          class="rounded-lg border-2 px-2 py-2.5 text-center font-display text-sm font-semibold transition-colors"
          :class="
            props.selectedSlot?.starts_at === slot.starts_at
              ? 'border-brand bg-brand text-white'
              : 'border-ink/10 bg-white hover:border-brand/60'
          "
          :aria-pressed="props.selectedSlot?.starts_at === slot.starts_at"
          @click="emit('selectSlot', slot)"
        >
          {{ slot.label }}
        </button>
      </div>
      <p class="mt-2 text-xs text-ink/50">
        Horários de {{ props.availability.timezone.replace(/_/g, ' ') }}
      </p>
    </div>
  </div>
</template>
