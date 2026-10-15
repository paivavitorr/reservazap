<script setup lang="ts">
import type { AgendamentoPainel } from '~/types/painel'

definePageMeta({ layout: 'painel', middleware: 'auth' })

const store = useAuthStore()
const api = useApi()
store.init(api)

const de = ref(dataLocal(new Date()))
const ate = ref('')
const itens = ref<AgendamentoPainel[]>([])
const erro = ref('')
const carregando = ref(false)

async function carregar() {
  carregando.value = true
  erro.value = ''
  try {
    const query: Record<string, string> = {}
    if (de.value) query.de = de.value
    if (ate.value) query.ate = ate.value
    itens.value = await api<AgendamentoPainel[]>('/painel/agenda', { query })
  } catch {
    erro.value = 'Não foi possível carregar a agenda'
  } finally {
    carregando.value = false
  }
}

onMounted(async () => {
  await store.restaurar()
  if (!store.autenticado) return navigateTo('/painel/login')
  await carregar()
})

async function agir(id: string, acao: 'cancelar' | 'concluir' | 'no-show') {
  try {
    if (acao === 'cancelar') {
      const motivo = window.prompt('Motivo do cancelamento (opcional)') ?? undefined
      await api(`/painel/agendamentos/${id}/cancelar`, {
        method: 'POST',
        body: { reason: motivo || undefined },
      })
    } else {
      await api(`/painel/agendamentos/${id}/${acao}`, { method: 'POST' })
    }
    await carregar()
  } catch (e: unknown) {
    const err = e as { data?: { description?: string } }
    erro.value = err?.data?.description || 'Ação não permitida'
  }
}

/** YYYY-MM-DD no fuso local do navegador. */
function dataLocal(d: Date): string {
  const ano = d.getFullYear()
  const mes = String(d.getMonth() + 1).padStart(2, '0')
  const dia = String(d.getDate()).padStart(2, '0')
  return `${ano}-${mes}-${dia}`
}

function formatarHora(iso: string): string {
  return new Date(iso).toLocaleString('pt-BR', {
    day: '2-digit',
    month: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}
</script>

<template>
  <div>
    <h1 class="font-display text-2xl font-bold">Agenda</h1>

    <div class="mt-4 flex flex-wrap items-end gap-3">
      <div>
        <label for="de" class="mb-1 block text-sm font-medium">De</label>
        <input id="de" v-model="de" type="date" class="rounded-lg border border-ink/15 bg-white px-3 py-2.5">
      </div>
      <div>
        <label for="ate" class="mb-1 block text-sm font-medium">Até</label>
        <input id="ate" v-model="ate" type="date" class="rounded-lg border border-ink/15 bg-white px-3 py-2.5">
      </div>
      <button
        type="button"
        class="rounded-lg bg-brand px-4 py-2.5 font-semibold text-white transition-colors hover:bg-brand-dark"
        @click="carregar"
      >
        Filtrar
      </button>
    </div>

    <p v-if="erro" class="mt-4 rounded-lg bg-danger/10 px-4 py-2.5 text-sm text-danger">{{ erro }}</p>

    <p v-if="carregando" class="mt-6 text-sm text-ink/50">Carregando…</p>
    <p v-else-if="itens.length === 0" class="mt-6 rounded-lg bg-white px-4 py-8 text-center text-sm text-ink/55">
      Nenhum agendamento no período.
    </p>

    <ul v-else class="mt-6 grid gap-2">
      <li
        v-for="a in itens"
        :key="a.id"
        class="rounded-lg border border-ink/10 bg-white px-4 py-3"
      >
        <div class="flex flex-wrap items-start justify-between gap-3">
          <div>
            <p class="font-medium">
              {{ formatarHora(a.starts_at) }} · {{ a.client_name }}
              <span class="text-sm text-ink/50">· {{ a.client_phone }}</span>
            </p>
            <p class="text-sm text-ink/55">{{ a.service }} com {{ a.professional }}</p>
            <p class="text-xs text-ink/45">
              Sinal {{ a.signal_paid_at ? 'pago' : 'pendente' }} · {{ formatCents(a.signal_amount_in_cents) }}
            </p>
            <p v-if="a.cancellation_reason" class="text-xs text-danger">
              Cancelado: {{ a.cancellation_reason }}
            </p>
          </div>

          <div class="flex flex-wrap items-center gap-2">
            <span
              class="rounded-full px-2.5 py-1 text-xs font-semibold"
              :class="{
                'bg-amber/15 text-amber': a.status === 'hold',
                'bg-brand-soft text-brand': a.status === 'confirmed',
                'bg-ink/10 text-ink/60': a.status === 'completed',
                'bg-danger/10 text-danger': a.status === 'cancelled' || a.status === 'no_show' || a.status === 'expired',
              }"
            >
              {{ a.status }}
            </span>

            <template v-if="a.status === 'confirmed'">
              <button
                type="button"
                class="rounded border border-brand px-2.5 py-1 text-xs font-semibold text-brand hover:bg-brand-soft"
                @click="agir(a.id, 'concluir')"
              >
                Concluir
              </button>
              <button
                type="button"
                class="rounded border border-amber px-2.5 py-1 text-xs font-semibold text-amber hover:bg-amber/10"
                @click="agir(a.id, 'no-show')"
              >
                No-show
              </button>
            </template>
            <button
              v-if="a.status === 'confirmed' || a.status === 'hold'"
              type="button"
              class="rounded border border-danger px-2.5 py-1 text-xs font-semibold text-danger hover:bg-danger/10"
              @click="agir(a.id, 'cancelar')"
            >
              Cancelar
            </button>
          </div>
        </div>
      </li>
    </ul>
  </div>
</template>
