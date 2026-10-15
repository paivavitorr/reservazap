<script setup lang="ts">
const props = defineProps<{ atual: number }>()
const etapas = ['Serviço', 'Profissional', 'Horário', 'Dados', 'Pagamento']
</script>

<template>
  <ol class="flex flex-wrap items-center gap-x-2 gap-y-1 text-xs text-ink/50" aria-label="Etapas do agendamento">
    <li v-for="(etapa, i) in etapas" :key="etapa" class="flex items-center gap-2">
      <span
        class="flex items-center gap-1.5"
        :class="i + 1 === props.atual ? 'font-semibold text-brand' : i + 1 < props.atual ? 'text-ink/70' : ''"
      >
        <span
          class="flex h-5 w-5 items-center justify-center rounded-full text-[10px] font-semibold"
          :class="
            i + 1 === props.atual
              ? 'bg-brand text-white'
              : i + 1 < props.atual
                ? 'bg-brand-soft text-brand'
                : 'bg-ink/10 text-ink/50'
          "
        >
          {{ i + 1 }}
        </span>
        {{ etapa }}
      </span>
      <span v-if="i < etapas.length - 1" aria-hidden="true">·</span>
    </li>
  </ol>
</template>
