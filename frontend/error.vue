<script setup lang="ts">
const error = useError()

function recarregar() {
  clearError({ redirect: '/' })
}
</script>

<template>
  <div class="flex min-h-screen items-center justify-center bg-paper px-6">
    <div class="w-full max-w-md text-center">
      <p class="font-display text-6xl font-bold text-brand">{{ error?.statusCode ?? 'Ops' }}</p>
      <h1 class="mt-3 text-xl font-semibold">
        {{ error?.statusCode === 404 ? 'Página não encontrada' : 'Algo deu errado' }}
      </h1>
      <p class="mt-2 text-ink/60">
        {{ error?.statusCode === 404
          ? 'O link pode estar errado ou o estabelecimento não está disponível.'
          : 'Tente novamente em instantes.' }}
      </p>
      <button
        type="button"
        class="mt-6 rounded-lg bg-brand px-6 py-3 font-semibold text-white transition-colors hover:bg-brand-dark"
        @click="recarregar"
      >
        Voltar ao início
      </button>
    </div>
  </div>
</template>
