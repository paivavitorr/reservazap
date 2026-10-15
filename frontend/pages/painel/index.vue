<script setup lang="ts">
import type { AgendamentoPainel, ResumoPainel } from '~/types/painel'

definePageMeta({ layout: 'painel', middleware: 'auth' })

const store = useAuthStore()
const api = useApi()
store.init(api)

const resumo = ref<ResumoPainel | null>(null)
const hoje = ref<AgendamentoPainel[]>([])
const erro = ref('')

onMounted(async () => {
  await store.restaurar()
  if (!store.autenticado) return navigateTo('/painel/login')
  try {
    const hojeStr = dataLocal(new Date())
    const [r, agenda] = await Promise.all([
      api<ResumoPainel>('/painel/resumo'),
      api<AgendamentoPainel[]>('/painel/agenda', { query: { de: hojeStr, ate: hojeStr } }),
    ])
    resumo.value = r
    hoje.value = agenda
  } catch {
    erro.value = 'Não foi possível carregar o painel'
  }
})

/** YYYY-MM-DD no fuso local do navegador (evita o off-by-one do UTC). */
function dataLocal(d: Date): string {
  const ano = d.getFullYear()
  const mes = String(d.getMonth() + 1).padStart(2, '0')
  const dia = String(d.getDate()).padStart(2, '0')
  return `${ano}-${mes}-${dia}`
}

const cards = computed(() => [
  { rotulo: 'Atendimentos hoje', valor: resumo.value?.hoje ?? '—' },
  { rotulo: 'Confirmados hoje', valor: resumo.value?.confirmados_hoje ?? '—' },
  { rotulo: 'Na semana', valor: resumo.value?.semana ?? '—' },
  {
    rotulo: 'No-show (30d)',
    valor: resumo.value ? `${resumo.value.taxa_no_show_30d}%` : '—',
  },
  {
    rotulo: 'Sinais pagos (30d)',
    valor: resumo.value ? formatCents(resumo.value.sinais_pagos_30d) : '—',
  },
])
</script>

<template>
  <div>
    <h1 class="font-display text-2xl font-bold">Visão geral</h1>
    <p class="mt-1 text-sm text-ink/60">Seu negócio em um olhar.</p>

    <p v-if="erro" class="mt-4 rounded-lg bg-danger/10 px-4 py-2.5 text-sm text-danger">{{ erro }}</p>

    <div class="mt-6 grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-5">
      <div v-for="card in cards" :key="card.rotulo" class="rounded-lg border border-ink/10 bg-white px-4 py-4">
        <p class="text-xs text-ink/55">{{ card.rotulo }}</p>
        <p class="mt-1 font-display text-xl font-semibold">{{ card.valor }}</p>
      </div>
    </div>

    <section class="mt-8">
      <div class="flex items-center justify-between">
        <h2 class="font-display text-lg font-semibold">Agenda de hoje</h2>
        <NuxtLink to="/painel/agenda" class="text-sm font-semibold text-brand">Ver agenda completa</NuxtLink>
      </div>

      <p v-if="hoje.length === 0" class="mt-3 rounded-lg bg-white px-4 py-6 text-center text-sm text-ink/55">
        Nenhum atendimento marcado para hoje.
      </p>
      <ul v-else class="mt-3 grid gap-2">
        <li
          v-for="agendamento in hoje"
          :key="agendamento.id"
          class="flex flex-wrap items-center justify-between gap-2 rounded-lg border border-ink/10 bg-white px-4 py-3"
        >
          <div>
            <p class="font-medium">
              {{ new Date(agendamento.starts_at).toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit' }) }}
              · {{ agendamento.client_name }}
            </p>
            <p class="text-sm text-ink/55">{{ agendamento.service }} com {{ agendamento.professional }}</p>
          </div>
          <span
            class="rounded-full px-2.5 py-1 text-xs font-semibold"
            :class="{
              'bg-amber/15 text-amber': agendamento.status === 'hold',
              'bg-brand-soft text-brand': agendamento.status === 'confirmed',
              'bg-ink/10 text-ink/60': agendamento.status === 'completed',
              'bg-danger/10 text-danger': agendamento.status === 'cancelled' || agendamento.status === 'no_show' || agendamento.status === 'expired',
            }"
          >
            {{ agendamento.status }}
          </span>
        </li>
      </ul>
    </section>
  </div>
</template>
