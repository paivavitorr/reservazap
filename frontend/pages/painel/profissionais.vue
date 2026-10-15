<script setup lang="ts">
import type { ProfissionalPainel } from '~/types/painel'

definePageMeta({ layout: 'painel', middleware: 'auth' })

const store = useAuthStore()
const api = useApi()
store.init(api)

const itens = ref<ProfissionalPainel[]>([])
const erro = ref('')
const editando = ref<ProfissionalPainel | null>(null)
const form = ref({ name: '', email: '', phone: '' })

async function carregar() {
  try {
    itens.value = await api<ProfissionalPainel[]>('/painel/profissionais')
  } catch {
    erro.value = 'Não foi possível carregar os profissionais'
  }
}

onMounted(async () => {
  await store.restaurar()
  if (!store.autenticado) return navigateTo('/painel/login')
  await carregar()
})

function novo() {
  editando.value = null
  form.value = { name: '', email: '', phone: '' }
}

function editar(prof: ProfissionalPainel) {
  editando.value = prof
  form.value = { name: prof.name, email: prof.email ?? '', phone: prof.phone ?? '' }
}

async function salvar() {
  erro.value = ''
  const body = {
    name: form.value.name,
    email: form.value.email || null,
    phone: form.value.phone || null,
  }
  try {
    if (editando.value) {
      await api(`/painel/profissionais/${editando.value.id}`, { method: 'PATCH', body })
    } else {
      await api('/painel/profissionais', { method: 'POST', body })
    }
    novo()
    await carregar()
  } catch (e: unknown) {
    const err = e as { data?: { description?: string } }
    erro.value = err?.data?.description || 'Não foi possível salvar'
  }
}

async function desativar(prof: ProfissionalPainel) {
  if (!window.confirm(`Desativar "${prof.name}"?`)) return
  try {
    await api(`/painel/profissionais/${prof.id}`, { method: 'DELETE' })
    await carregar()
  } catch {
    erro.value = 'Não foi possível desativar'
  }
}
</script>

<template>
  <div>
    <div class="flex items-center justify-between">
      <h1 class="font-display text-2xl font-bold">Profissionais</h1>
      <button
        type="button"
        class="rounded-lg bg-brand px-4 py-2 text-sm font-semibold text-white hover:bg-brand-dark"
        @click="novo"
      >
        Novo profissional
      </button>
    </div>

    <form class="mt-5 rounded-lg border border-ink/10 bg-white px-4 py-4" @submit.prevent="salvar">
      <h2 class="font-semibold">{{ editando ? `Editar ${editando.name}` : 'Novo profissional' }}</h2>
      <div class="mt-3 grid gap-3 sm:grid-cols-3">
        <div>
          <label class="mb-1 block text-sm font-medium">Nome</label>
          <input v-model="form.name" required maxlength="120" class="w-full rounded-lg border border-ink/15 px-3 py-2.5">
        </div>
        <div>
          <label class="mb-1 block text-sm font-medium">E-mail</label>
          <input v-model="form.email" type="email" maxlength="160" class="w-full rounded-lg border border-ink/15 px-3 py-2.5">
        </div>
        <div>
          <label class="mb-1 block text-sm font-medium">WhatsApp</label>
          <input v-model="form.phone" type="tel" maxlength="20" class="w-full rounded-lg border border-ink/15 px-3 py-2.5">
        </div>
      </div>
      <p v-if="erro" class="mt-3 rounded-lg bg-danger/10 px-4 py-2.5 text-sm text-danger">{{ erro }}</p>
      <button type="submit" class="mt-4 rounded-lg bg-accent px-5 py-2.5 font-semibold text-white hover:bg-accent-dark">
        {{ editando ? 'Salvar alterações' : 'Adicionar profissional' }}
      </button>
    </form>

    <ul class="mt-6 grid gap-2">
      <li
        v-for="prof in itens"
        :key="prof.id"
        class="flex flex-wrap items-center justify-between gap-3 rounded-lg border border-ink/10 bg-white px-4 py-3"
        :class="{ 'opacity-50': !prof.is_active }"
      >
        <div>
          <p class="font-medium">{{ prof.name }} <span v-if="!prof.is_active" class="text-xs text-ink/45">(desativado)</span></p>
          <p class="text-sm text-ink/55">{{ prof.email || '—' }} · {{ prof.phone || '—' }}</p>
        </div>
        <div class="flex gap-2">
          <button type="button" class="rounded border border-ink/15 px-3 py-1.5 text-sm hover:border-ink/30" @click="editar(prof)">
            Editar
          </button>
          <button
            v-if="prof.is_active"
            type="button"
            class="rounded border border-danger px-3 py-1.5 text-sm text-danger hover:bg-danger/10"
            @click="desativar(prof)"
          >
            Desativar
          </button>
        </div>
      </li>
    </ul>
  </div>
</template>
