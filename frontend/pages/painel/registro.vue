<script setup lang="ts">
definePageMeta({ layout: 'painel', middleware: 'auth' })

const store = useAuthStore()
store.init(useApi())

const form = reactive({
  tenant_name: '',
  slug: '',
  phone: '',
  name: '',
  email: '',
  password: '',
})
const erro = ref('')
const carregando = ref(false)

watch(
  () => form.tenant_name,
  (nome) => {
    form.slug = nome
      .toLowerCase()
      .normalize('NFD')
      .replace(/[\u0300-\u036f]/g, '')
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/(^-|-$)/g, '')
  }
)

async function cadastrar() {
  carregando.value = true
  erro.value = ''
  try {
    await store.registrar({ ...form, phone: form.phone })
    navigateTo('/painel')
  } catch (e: unknown) {
    const err = e as { data?: { description?: string } }
    erro.value = err?.data?.description || 'Não foi possível cadastrar'
  } finally {
    carregando.value = false
  }
}
</script>

<template>
  <div class="mx-auto max-w-md">
    <h1 class="font-display text-2xl font-bold">Cadastre seu estabelecimento</h1>
    <p class="mt-1 text-sm text-ink/60">
      Em poucos minutos você tem seu link de agendamento com sinal via Pix.
    </p>

    <form class="mt-6 grid gap-3" @submit.prevent="cadastrar">
      <div>
        <label for="tenant_name" class="mb-1 block text-sm font-medium">Nome do estabelecimento</label>
        <input
          id="tenant_name"
          v-model="form.tenant_name"
          type="text"
          required
          maxlength="120"
          class="w-full rounded-lg border border-ink/15 bg-white px-3.5 py-3"
        >
      </div>
      <div>
        <label for="slug" class="mb-1 block text-sm font-medium">Seu link</label>
        <div class="flex items-center gap-1 rounded-lg border border-ink/15 bg-white px-3.5 py-3">
          <span class="text-sm text-ink/40">/</span>
          <input
            id="slug"
            v-model="form.slug"
            type="text"
            required
            maxlength="64"
            pattern="[a-z0-9\-]+"
            class="w-full bg-transparent outline-none"
          >
        </div>
      </div>
      <div>
        <label for="phone" class="mb-1 block text-sm font-medium">WhatsApp</label>
        <input
          id="phone"
          v-model="form.phone"
          type="tel"
          required
          placeholder="(11) 98765-4321"
          class="w-full rounded-lg border border-ink/15 bg-white px-3.5 py-3"
        >
      </div>
      <div>
        <label for="name" class="mb-1 block text-sm font-medium">Seu nome</label>
        <input
          id="name"
          v-model="form.name"
          type="text"
          required
          maxlength="120"
          class="w-full rounded-lg border border-ink/15 bg-white px-3.5 py-3"
        >
      </div>
      <div>
        <label for="email" class="mb-1 block text-sm font-medium">E-mail</label>
        <input
          id="email"
          v-model="form.email"
          type="email"
          required
          class="w-full rounded-lg border border-ink/15 bg-white px-3.5 py-3"
        >
      </div>
      <div>
        <label for="password" class="mb-1 block text-sm font-medium">Senha <span class="text-ink/40">(mín. 8)</span></label>
        <input
          id="password"
          v-model="form.password"
          type="password"
          required
          minlength="8"
          autocomplete="new-password"
          class="w-full rounded-lg border border-ink/15 bg-white px-3.5 py-3"
        >
      </div>

      <p v-if="erro" class="rounded-lg bg-danger/10 px-4 py-2.5 text-sm text-danger">{{ erro }}</p>

      <button
        type="submit"
        :disabled="carregando"
        class="rounded-lg bg-accent px-5 py-3.5 font-semibold text-white transition-colors hover:bg-accent-dark disabled:opacity-50"
      >
        {{ carregando ? 'Criando conta…' : 'Criar conta e começar' }}
      </button>
    </form>

    <p class="mt-4 text-sm text-ink/60">
      Já tem conta?
      <NuxtLink to="/painel/login" class="font-semibold text-brand">Entrar</NuxtLink>
    </p>
  </div>
</template>
