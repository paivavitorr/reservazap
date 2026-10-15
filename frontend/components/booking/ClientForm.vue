<script setup lang="ts">
const nome = defineModel<string>('nome', { required: true })
const telefone = defineModel<string>('telefone', { required: true })
const email = defineModel<string>('email', { required: true })

function onTelefone(e: Event) {
  telefone.value = maskPhone((e.target as HTMLInputElement).value)
}
</script>

<template>
  <div class="grid gap-3">
    <div>
      <label for="client-name" class="mb-1 block text-sm font-medium">Nome completo</label>
      <input
        id="client-name"
        v-model="nome"
        type="text"
        autocomplete="name"
        maxlength="120"
        class="w-full rounded-lg border border-ink/15 bg-white px-3.5 py-3"
        placeholder="Como devemos te chamar"
      >
    </div>
    <div>
      <label for="client-phone" class="mb-1 block text-sm font-medium">WhatsApp</label>
      <input
        id="client-phone"
        :value="telefone"
        type="tel"
        inputmode="numeric"
        autocomplete="tel"
        class="w-full rounded-lg border border-ink/15 bg-white px-3.5 py-3"
        placeholder="(11) 98765-4321"
        @input="onTelefone"
      >
    </div>
    <div>
      <label for="client-email" class="mb-1 block text-sm font-medium">E-mail <span class="text-ink/40">(opcional)</span></label>
      <input
        id="client-email"
        v-model="email"
        type="email"
        autocomplete="email"
        maxlength="160"
        class="w-full rounded-lg border border-ink/15 bg-white px-3.5 py-3"
        placeholder="voce@email.com"
      >
    </div>
  </div>
</template>
