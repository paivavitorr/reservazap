<script setup lang="ts">
import type { BookingIntentResponse } from '~/types/booking'

const props = defineProps<{
  hold: BookingIntentResponse
  remainingSeconds: number
}>()
const emit = defineEmits<{ expired: [] }>()

const copiado = ref(false)
const total = computed(() => props.hold.signal.pix.expiracao_segundos || 600)
const progresso = computed(() => Math.min(1, props.remainingSeconds / total.value))
const corBarra = computed(() => {
  if (progresso.value > 0.5) return 'bg-brand'
  if (progresso.value > 0.2) return 'bg-amber'
  return 'bg-danger'
})

watch(
  () => props.remainingSeconds,
  (s) => {
    if (s <= 0) emit('expired')
  }
)

async function copiarPix() {
  const codigo = props.hold.signal.pix.copia_e_cola
  try {
    await navigator.clipboard.writeText(codigo)
    copiado.value = true
    setTimeout(() => (copiado.value = false), 2500)
  } catch {
    copiado.value = false
  }
}
</script>

<template>
  <div class="overflow-hidden rounded-lg border border-ink/10 bg-white">
    <!-- Barra de tempo restante -->
    <div class="h-1.5 w-full bg-ink/10">
      <div
        class="h-full transition-[width] duration-1000 ease-linear"
        :class="corBarra"
        :style="{ width: `${progresso * 100}%` }"
      />
    </div>

    <div class="px-5 py-5 text-center">
      <p class="text-sm text-ink/55">Valor do sinal</p>
      <p class="font-display text-3xl font-bold text-brand">
        {{ formatCents(props.hold.signal.amount_in_cents) }}
      </p>
      <p class="mt-1 text-sm text-ink/60">
        Tempo restante:
        <span class="font-display font-semibold" :class="progresso <= 0.2 ? 'text-danger' : 'text-ink'">
          {{ formatCountdown(props.remainingSeconds) }}
        </span>
      </p>
    </div>

    <!-- QR Code -->
    <div v-if="props.hold.signal.pix.qr_code_base64" class="border-t border-dashed border-ink/20 px-5 py-5">
      <p class="mb-3 text-sm font-medium">Aponte a câmera do seu banco para o QR Code</p>
      <div class="mx-auto w-fit rounded-md border border-ink/10 bg-white p-2">
        <img
          :src="props.hold.signal.pix.qr_code_base64"
          alt="QR Code Pix"
          class="h-48 w-48"
        >
      </div>
    </div>

    <!-- Copia e cola -->
    <div class="border-t border-dashed border-ink/20 px-5 py-5">
      <p class="mb-2 text-sm font-medium">Ou use o Pix copia e cola</p>
      <input
        :value="props.hold.signal.pix.copia_e_cola"
        readonly
        class="w-full rounded border border-ink/15 bg-paper px-2.5 py-2 font-mono text-xs text-ink/70"
        aria-label="Código Pix copia e cola"
      >
      <button
        type="button"
        class="mt-2 w-full rounded-lg bg-ink px-4 py-2.5 text-sm font-semibold text-white transition-colors hover:bg-ink/85"
        @click="copiarPix"
      >
        {{ copiado ? 'Código copiado!' : 'Copiar código Pix' }}
      </button>
    </div>
  </div>
</template>
