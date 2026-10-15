<script setup lang="ts">
import type { Professional } from '~/types/booking'

const props = defineProps<{ professional: Professional; selected: boolean }>()
const emit = defineEmits<{ select: [] }>()
</script>

<template>
  <button
    type="button"
    class="w-full rounded-lg border-2 bg-white px-4 py-3 text-left font-medium transition-colors"
    :class="props.selected ? 'border-brand bg-brand-soft/40' : 'border-ink/10 hover:border-ink/25'"
    :aria-pressed="props.selected"
    @click="emit('select')"
  >
    {{ props.professional.name }}
  </button>
</template>
