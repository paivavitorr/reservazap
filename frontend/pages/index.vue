<script setup lang="ts">
useSeoMeta({
  title: 'Agendamento online com sinal via Pix',
  description:
    'Seu cliente agenda pelo link, paga o sinal por Pix e o horário fica garantido. Sem app, sem complicação.',
  ogTitle: 'ReservaZap · Agendamento online com sinal via Pix',
  ogDescription:
    'Seu cliente agenda pelo link, paga o sinal por Pix e o horário fica garantido.',
  ogType: 'website',
})

const etapas = [
  {
    titulo: 'O cliente escolhe o horário',
    texto: 'Um link no WhatsApp mostra seus serviços, profissionais e horários livres de verdade.',
  },
  {
    titulo: 'Ele paga o sinal por Pix',
    texto: 'QR Code e copia e cola na tela. O horário fica reservado por 10 minutos enquanto o pagamento cai.',
  },
  {
    titulo: 'Falta zero',
    texto: 'Sinal pago, horário garantido. Quem desmarcar cedo libera a vaga para outro cliente.',
  },
]
</script>

<template>
  <div class="min-h-screen bg-paper">
    <header class="border-b border-ink/10 bg-white">
      <div class="mx-auto flex max-w-3xl items-center justify-between px-6 py-4">
        <p class="font-display text-lg font-bold text-brand">ReservaZap</p>
        <a
          href="/barbearia-do-zeu"
          class="rounded-lg border border-brand px-4 py-2 text-sm font-semibold text-brand transition-colors hover:bg-brand-soft"
        >
          Ver demonstração
        </a>
      </div>
    </header>

    <main class="mx-auto max-w-3xl px-6">
      <section class="py-14">
        <h1 class="max-w-2xl font-display text-4xl font-bold leading-tight sm:text-5xl">
          Horário marcado com
          <span class="text-brand">sinal pago</span>
          não tem falta.
        </h1>
        <p class="mt-5 max-w-xl text-lg leading-relaxed text-ink/70">
          Um link de agendamento para pequenos prestadores de serviço. O cliente escolhe o horário,
          paga o sinal por Pix na hora e você para de perder a agenda com desistência de última hora.
        </p>
        <div class="mt-8 flex flex-wrap gap-3">
          <a
            href="/barbearia-do-zeu"
            class="rounded-lg bg-accent px-6 py-3.5 font-display font-semibold text-white transition-colors hover:bg-accent-dark"
          >
            Agendar na demo
          </a>
          <a
            href="#como-funciona"
            class="rounded-lg border border-ink/15 px-6 py-3.5 font-semibold transition-colors hover:border-ink/30"
          >
            Como funciona
          </a>
        </div>
      </section>

      <section id="como-funciona" class="border-t border-ink/10 py-12">
        <h2 class="font-display text-2xl font-semibold">Como funciona</h2>
        <ol class="mt-6 grid gap-6 sm:grid-cols-3">
          <li v-for="(etapa, i) in etapas" :key="etapa.titulo">
            <p class="font-display text-sm font-semibold text-brand">{{ i + 1 }}</p>
            <h3 class="mt-1 font-semibold leading-snug">{{ etapa.titulo }}</h3>
            <p class="mt-1.5 text-sm leading-relaxed text-ink/65">{{ etapa.texto }}</p>
          </li>
        </ol>
      </section>

      <section class="border-t border-ink/10 py-12">
        <div class="rounded-lg border border-brand/25 bg-brand-soft/50 px-6 py-8 sm:flex sm:items-center sm:justify-between">
          <div>
            <h2 class="font-display text-xl font-semibold">Quer um link assim para o seu negócio?</h2>
            <p class="mt-1 text-sm text-ink/65">
              Serviços, profissionais e horários sob seu controle, com Pix integrado.
            </p>
          </div>
          <a
            href="/barbearia-do-zeu"
            class="mt-4 inline-block rounded-lg bg-brand px-5 py-3 font-semibold text-white transition-colors hover:bg-brand-dark sm:mt-0"
          >
            Conhecer a demo
          </a>
        </div>
      </section>
    </main>

    <footer class="border-t border-ink/10 px-6 py-8 text-center text-sm text-ink/50">
      ReservaZap · agendamentos com sinal via Pix
    </footer>
  </div>
</template>
