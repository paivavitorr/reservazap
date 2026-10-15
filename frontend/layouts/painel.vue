<script setup lang="ts">
const store = useAuthStore()
store.init(useApi())

onMounted(async () => {
  await store.restaurar()
  if (!store.autenticado) {
    navigateTo('/painel/login')
  }
})

const rotas = [
  { para: '/painel', texto: 'Início' },
  { para: '/painel/agenda', texto: 'Agenda' },
  { para: '/painel/servicos', texto: 'Serviços' },
  { para: '/painel/profissionais', texto: 'Profissionais' },
  { para: '/painel/expediente', texto: 'Expediente' },
]

function sair() {
  store.logout()
  navigateTo('/painel/login')
}
</script>

<template>
  <div class="min-h-screen bg-paper">
    <header class="border-b border-ink/10 bg-white">
      <div class="mx-auto flex max-w-5xl items-center justify-between gap-4 px-5 py-3">
        <div class="flex items-center gap-3">
          <NuxtLink to="/painel" class="font-display text-lg font-bold text-brand">ReservaZap</NuxtLink>
          <span class="hidden text-xs text-ink/45 sm:inline">painel</span>
        </div>
        <nav class="flex flex-wrap items-center gap-1 text-sm">
          <NuxtLink
            v-for="rota in rotas"
            :key="rota.para"
            :to="rota.para"
            class="rounded px-2.5 py-1.5 text-ink/70 transition-colors hover:bg-brand-soft hover:text-brand"
            active-class="bg-brand-soft font-semibold text-brand"
          >
            {{ rota.texto }}
          </NuxtLink>
          <button
            type="button"
            class="ml-2 rounded px-2.5 py-1.5 text-ink/50 transition-colors hover:text-danger"
            @click="sair"
          >
            Sair
          </button>
        </nav>
      </div>
    </header>

    <main class="mx-auto max-w-5xl px-5 py-6">
      <slot />
    </main>
  </div>
</template>
