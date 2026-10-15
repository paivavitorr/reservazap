<script setup lang="ts">
definePageMeta({ layout: 'painel', middleware: 'auth' })

const store = useAuthStore()
store.init(useApi())

const email = ref('')
const senha = ref('')
const erro = ref('')
const carregando = ref(false)

async function entrar() {
  carregando.value = true
  erro.value = ''
  try {
    await store.login(email.value.trim(), senha.value)
    navigateTo('/painel')
  } catch (e: unknown) {
    const err = e as { data?: { description?: string } }
    erro.value = err?.data?.description || 'Não foi possível entrar'
  } finally {
    carregando.value = false
  }
}
</script>

<template>
  <div class="mx-auto max-w-md">
    <h1 class="font-display text-2xl font-bold">Entrar no painel</h1>
    <p class="mt-1 text-sm text-ink/60">Gerencie sua agenda, serviços e horários.</p>

    <form class="mt-6 grid gap-3" @submit.prevent="entrar">
      <div>
        <label for="email" class="mb-1 block text-sm font-medium">E-mail</label>
        <input
          id="email"
          v-model="email"
          type="email"
          autocomplete="email"
          required
          class="w-full rounded-lg border border-ink/15 bg-white px-3.5 py-3"
        >
      </div>
      <div>
        <label for="senha" class="mb-1 block text-sm font-medium">Senha</label>
        <input
          id="senha"
          v-model="senha"
          type="password"
          autocomplete="current-password"
          required
          class="w-full rounded-lg border border-ink/15 bg-white px-3.5 py-3"
        >
      </div>

      <p v-if="erro" class="rounded-lg bg-danger/10 px-4 py-2.5 text-sm text-danger">{{ erro }}</p>

      <button
        type="submit"
        :disabled="carregando"
        class="rounded-lg bg-brand px-5 py-3.5 font-semibold text-white transition-colors hover:bg-brand-dark disabled:opacity-50"
      >
        {{ carregando ? 'Entrando…' : 'Entrar' }}
      </button>
    </form>

    <p class="mt-4 text-sm text-ink/60">
      Ainda não tem conta?
      <NuxtLink to="/painel/registro" class="font-semibold text-brand">Cadastre seu estabelecimento</NuxtLink>
    </p>
  </div>
</template>
