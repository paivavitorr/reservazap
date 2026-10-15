<script setup lang="ts">
import type { FaixaExpediente } from '~/types/painel'

definePageMeta({ layout: 'painel', middleware: 'auth' })

const store = useAuthStore()
const api = useApi()
store.init(api)

const dias = [
  'segunda-feira',
  'terça-feira',
  'quarta-feira',
  'quinta-feira',
  'sexta-feira',
  'sábado',
  'domingo',
]

const faixas = ref<FaixaExpediente[]>([])
const erro = ref('')
const salvo = ref(false)

async function carregar() {
  try {
    faixas.value = await api<FaixaExpediente[]>('/painel/expediente')
  } catch {
    erro.value = 'Não foi possível carregar o expediente'
  }
}

onMounted(async () => {
  await store.restaurar()
  if (!store.autenticado) return navigateTo('/painel/login')
  await carregar()
})

const porDia = computed(() =>
  dias.map((_dia, weekday) => ({
    weekday,
    faixas: faixas.value
      .filter((f) => f.weekday === weekday)
      .sort((a, b) => a.start.localeCompare(b.start)),
  }))
)

function adicionar(weekday: number) {
  faixas.value.push({ weekday, start: '09:00', end: '18:00' })
  salvo.value = false
}

function remover(faixa: FaixaExpediente) {
  faixas.value = faixas.value.filter((f) => f !== faixa)
  salvo.value = false
}

async function salvar() {
  erro.value = ''
  salvo.value = false
  try {
    faixas.value = await api<FaixaExpediente[]>('/painel/expediente', {
      method: 'PUT',
      body: { items: faixas.value.map((f) => ({ weekday: f.weekday, start: f.start, end: f.end })) },
    })
    salvo.value = true
    setTimeout(() => (salvo.value = false), 2500)
  } catch (e: unknown) {
    const err = e as { data?: { description?: string } }
    erro.value = err?.data?.description || 'Não foi possível salvar'
  }
}
</script>

<template>
  <div>
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div>
        <h1 class="font-display text-2xl font-bold">Expediente</h1>
        <p class="mt-1 text-sm text-ink/60">Quando seu estabelecimento atende. Os horários aparecem para o cliente nessa janela.</p>
      </div>
      <button
        type="button"
        class="rounded-lg bg-brand px-5 py-2.5 font-semibold text-white transition-colors hover:bg-brand-dark"
        @click="salvar"
      >
        Salvar expediente
      </button>
    </div>

    <p v-if="erro" class="mt-4 rounded-lg bg-danger/10 px-4 py-2.5 text-sm text-danger">{{ erro }}</p>
    <p v-if="salvo" class="mt-4 rounded-lg bg-brand-soft px-4 py-2.5 text-sm text-brand">Expediente salvo!</p>

    <div class="mt-6 grid gap-3">
      <div
        v-for="dia in porDia"
        :key="dia.weekday"
        class="rounded-lg border border-ink/10 bg-white px-4 py-3"
      >
        <div class="flex flex-wrap items-center justify-between gap-2">
          <p class="font-medium">{{ dias[dia.weekday] }}</p>
          <button
            type="button"
            class="rounded border border-brand px-2.5 py-1 text-xs font-semibold text-brand hover:bg-brand-soft"
            @click="adicionar(dia.weekday)"
          >
            + faixa
          </button>
        </div>

        <p v-if="dia.faixas.length === 0" class="mt-1 text-sm text-ink/45">Fechado</p>

        <div v-for="faixa in dia.faixas" :key="`${faixa.weekday}-${faixa.start}-${faixa.end}`" class="mt-2 flex items-center gap-2">
          <input
            v-model="faixa.start"
            type="time"
            class="rounded-lg border border-ink/15 px-2.5 py-2"
            @change="salvo = false"
          >
          <span class="text-ink/40">até</span>
          <input
            v-model="faixa.end"
            type="time"
            class="rounded-lg border border-ink/15 px-2.5 py-2"
            @change="salvo = false"
          >
          <button
            type="button"
            class="ml-auto rounded border border-danger px-2.5 py-1.5 text-xs text-danger hover:bg-danger/10"
            @click="remover(faixa)"
          >
            remover
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
