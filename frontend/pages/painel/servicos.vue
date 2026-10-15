<script setup lang="ts">
import type { ServicoPainel } from '~/types/painel'

definePageMeta({ layout: 'painel', middleware: 'auth' })

const store = useAuthStore()
const api = useApi()
store.init(api)

const itens = ref<ServicoPainel[]>([])
const erro = ref('')
const editando = ref<ServicoPainel | null>(null)
const vazio: Omit<ServicoPainel, 'id' | 'is_active'> = {
  name: '',
  description: '',
  duration_minutes: 30,
  price_in_cents: 0,
  signal_percent: 30,
}
const form = ref({ ...vazio })

async function carregar() {
  try {
    itens.value = await api<ServicoPainel[]>('/painel/servicos')
  } catch {
    erro.value = 'Não foi possível carregar os serviços'
  }
}

onMounted(async () => {
  await store.restaurar()
  if (!store.autenticado) return navigateTo('/painel/login')
  await carregar()
})

function novo() {
  editando.value = null
  form.value = { ...vazio }
}

function editar(servico: ServicoPainel) {
  editando.value = servico
  form.value = {
    name: servico.name,
    description: servico.description ?? '',
    duration_minutes: servico.duration_minutes,
    price_in_cents: servico.price_in_cents,
    signal_percent: servico.signal_percent,
  }
}

async function salvar() {
  erro.value = ''
  const body = {
    name: form.value.name,
    description: form.value.description || null,
    duration_minutes: Number(form.value.duration_minutes),
    price_in_cents: Number(form.value.price_in_cents),
    signal_percent: Number(form.value.signal_percent),
  }
  try {
    if (editando.value) {
      await api(`/painel/servicos/${editando.value.id}`, { method: 'PATCH', body })
    } else {
      await api('/painel/servicos', { method: 'POST', body })
    }
    novo()
    await carregar()
  } catch (e: unknown) {
    const err = e as { data?: { description?: string } }
    erro.value = err?.data?.description || 'Não foi possível salvar'
  }
}

async function desativar(servico: ServicoPainel) {
  if (!window.confirm(`Desativar o serviço "${servico.name}"?`)) return
  try {
    await api(`/painel/servicos/${servico.id}`, { method: 'DELETE' })
    await carregar()
  } catch {
    erro.value = 'Não foi possível desativar'
  }
}
</script>

<template>
  <div>
    <div class="flex items-center justify-between">
      <h1 class="font-display text-2xl font-bold">Serviços</h1>
      <button
        type="button"
        class="rounded-lg bg-brand px-4 py-2 text-sm font-semibold text-white hover:bg-brand-dark"
        @click="novo"
      >
        Novo serviço
      </button>
    </div>

    <form class="mt-5 rounded-lg border border-ink/10 bg-white px-4 py-4" @submit.prevent="salvar">
      <h2 class="font-semibold">{{ editando ? `Editar ${editando.name}` : 'Novo serviço' }}</h2>
      <div class="mt-3 grid gap-3 sm:grid-cols-2">
        <div>
          <label class="mb-1 block text-sm font-medium">Nome</label>
          <input v-model="form.name" required maxlength="120" class="w-full rounded-lg border border-ink/15 px-3 py-2.5">
        </div>
        <div>
          <label class="mb-1 block text-sm font-medium">Descrição</label>
          <input v-model="form.description" maxlength="255" class="w-full rounded-lg border border-ink/15 px-3 py-2.5">
        </div>
        <div>
          <label class="mb-1 block text-sm font-medium">Duração (min)</label>
          <input v-model.number="form.duration_minutes" type="number" min="5" max="1440" required class="w-full rounded-lg border border-ink/15 px-3 py-2.5">
        </div>
        <div>
          <label class="mb-1 block text-sm font-medium">Preço (R$)</label>
          <input
            :value="(form.price_in_cents / 100).toFixed(2)"
            type="number"
            min="0"
            step="0.01"
            required
            class="w-full rounded-lg border border-ink/15 px-3 py-2.5"
            @input="form.price_in_cents = Math.round(Number(($event.target as HTMLInputElement).value) * 100)"
          >
        </div>
        <div>
          <label class="mb-1 block text-sm font-medium">Sinal (%)</label>
          <input v-model.number="form.signal_percent" type="number" min="0" max="100" required class="w-full rounded-lg border border-ink/15 px-3 py-2.5">
        </div>
      </div>
      <p v-if="erro" class="mt-3 rounded-lg bg-danger/10 px-4 py-2.5 text-sm text-danger">{{ erro }}</p>
      <button type="submit" class="mt-4 rounded-lg bg-accent px-5 py-2.5 font-semibold text-white hover:bg-accent-dark">
        {{ editando ? 'Salvar alterações' : 'Adicionar serviço' }}
      </button>
    </form>

    <ul class="mt-6 grid gap-2">
      <li
        v-for="servico in itens"
        :key="servico.id"
        class="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-ink/10 bg-white px-4 py-3"
        :class="{ 'opacity-50': !servico.is_active }"
      >
        <div>
          <p class="font-medium">{{ servico.name }} <span v-if="!servico.is_active" class="text-xs text-ink/45">(desativado)</span></p>
          <p class="text-sm text-ink/55">
            {{ servico.duration_minutes }} min · {{ formatCents(servico.price_in_cents) }} · sinal {{ servico.signal_percent }}%
          </p>
        </div>
        <div class="flex gap-2">
          <button type="button" class="rounded border border-ink/15 px-3 py-1.5 text-sm hover:border-ink/30" @click="editar(servico)">
            Editar
          </button>
          <button
            v-if="servico.is_active"
            type="button"
            class="rounded border border-danger px-3 py-1.5 text-sm text-danger hover:bg-danger/10"
            @click="desativar(servico)"
          >
            Desativar
          </button>
        </div>
      </li>
    </ul>
  </div>
</template>
