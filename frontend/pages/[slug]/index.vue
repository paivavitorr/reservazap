<script setup lang="ts">
const route = useRoute()
const slug = route.params.slug as string

const store = useBookingStore()
store.init(useApi())

await useAsyncData(`catalog-${slug}`, () => store.fetchCatalog(slug))

useSeoMeta({
  title: () => (store.catalog ? `Agende na ${store.catalog.tenant.name}` : 'ReservaZap'),
  description: () =>
    store.catalog
      ? `Agende seu horário na ${store.catalog.tenant.name} e garanta seu atendimento com o sinal via Pix.`
      : 'Agendamentos online com sinal via Pix.',
  ogTitle: () => (store.catalog ? `${store.catalog.tenant.name} · Agendamento online` : 'ReservaZap'),
  ogDescription: () =>
    store.catalog
      ? `Escolha o serviço, o horário e garanta seu atendimento na ${store.catalog.tenant.name}.`
      : 'Agendamentos online com sinal via Pix.',
  ogType: 'website',
})

// Carrega os horários quando o profissional é escolhido
watch(
  () => store.selectedProfessional,
  (prof) => {
    if (prof) store.fetchAvailability(slug)
  }
)

const etapaAtual = computed(() => {
  if (store.step === 'hold') return 5
  if (!store.selectedService) return 1
  if (!store.selectedProfessional) return 2
  if (!store.selectedSlot) return 3
  return 4
})

// Contador regressivo do hold
const remaining = ref(0)
let tickInterval: ReturnType<typeof setInterval> | null = null
let statusInterval: ReturnType<typeof setInterval> | null = null

function pararTimers() {
  if (tickInterval) clearInterval(tickInterval)
  if (statusInterval) clearInterval(statusInterval)
  tickInterval = null
  statusInterval = null
}

function iniciarTimers() {
  pararTimers()
  remaining.value = store.holdRemainingSeconds
  tickInterval = setInterval(() => {
    remaining.value = store.holdRemainingSeconds
    if (remaining.value <= 0) pararTimers()
  }, 1000)
  statusInterval = setInterval(() => store.checkStatus(slug), 3000)
}

watch(
  () => store.step,
  (step) => {
    if (step === 'hold') iniciarTimers()
    else pararTimers()
  }
)

onUnmounted(pararTimers)

function reiniciar() {
  pararTimers()
  store.reset()
  store.fetchCatalog(slug)
}

const linkWhats = computed(() => {
  if (!store.catalog) return ''
  const digitos = store.catalog.tenant.phone?.replace(/\D/g, '')
  if (!digitos) return ''
  // 10-11 dígitos é número local (DD+numero); 12-13 já incluem o DDI 55
  const telefone = digitos.length <= 11 ? `55${digitos}` : digitos
  const texto = encodeURIComponent(
    `Olá! Confirmei meu agendamento na ${store.catalog.tenant.name} pelo ReservaZap.`
  )
  return `https://wa.me/${telefone}?text=${texto}`
})

function baixarCalendario() {
  if (!store.catalog || !store.selectedService || !store.selectedSlot) return
  const ics = gerarIcs({
    uid: store.hold?.appointment_id ?? store.selectedSlot.starts_at,
    titulo: `${store.selectedService.name} · ${store.catalog.tenant.name}`,
    descricao: `Agendamento confirmado via ReservaZap. Profissional: ${store.selectedProfessional?.name ?? ''}.`,
    local: store.catalog.tenant.name,
    inicioUtc: store.selectedSlot.starts_at,
    duracaoMinutos: store.selectedService.duration_minutes,
  })
  baixarArquivo('agendamento.ics', ics)
}
</script>

<template>
  <div class="min-h-screen bg-paper">
    <!-- Carregando -->
    <div v-if="store.loading && !store.catalog" class="flex min-h-screen items-center justify-center">
      <p class="text-ink/50">Carregando…</p>
    </div>

    <!-- Estabelecimento não encontrado -->
    <div v-else-if="store.error && !store.catalog" class="flex min-h-screen items-center justify-center px-6">
      <div class="w-full max-w-md rounded-lg border border-ink/10 bg-white px-6 py-8 text-center">
        <p class="font-display text-4xl font-bold text-danger">404</p>
        <h1 class="mt-2 text-lg font-semibold">Estabelecimento não encontrado</h1>
        <p class="mt-2 text-sm text-ink/60">{{ store.error }}</p>
      </div>
    </div>

    <template v-else-if="store.catalog">
      <header class="border-b border-ink/10 bg-white">
        <div class="mx-auto max-w-lg px-5 py-5">
          <h1 class="font-display text-2xl font-bold leading-tight">{{ store.catalog.tenant.name }}</h1>
          <p class="mt-0.5 text-sm text-ink/55">Agende seu horário em poucos passos</p>
          <div v-if="store.step !== 'confirmed' && store.step !== 'expired'" class="mt-3">
            <BookingStepRail :atual="etapaAtual" />
          </div>
        </div>
      </header>

      <main class="mx-auto max-w-lg px-5 py-6">
        <!-- Fluxo de agendamento -->
        <template v-if="store.step === 'catalog' || store.step === 'selecting'">
          <section class="mb-7">
            <h2 class="mb-3 font-display text-lg font-semibold">Serviço</h2>
            <div class="grid gap-2.5">
              <BookingServiceCard
                v-for="service in store.catalog.services"
                :key="service.id"
                :service="service"
                :selected="store.selectedService?.id === service.id"
                @select="store.selectService(service)"
              />
            </div>
          </section>

          <section v-if="store.selectedService" class="mb-7">
            <h2 class="mb-3 font-display text-lg font-semibold">Profissional</h2>
            <div class="grid gap-2.5">
              <BookingProfessionalCard
                v-for="prof in store.catalog.professionals"
                :key="prof.id"
                :professional="prof"
                :selected="store.selectedProfessional?.id === prof.id"
                @select="store.selectProfessional(prof)"
              />
            </div>
          </section>

          <section v-if="store.selectedProfessional" class="mb-7">
            <h2 class="mb-3 font-display text-lg font-semibold">Horário</h2>
            <p v-if="store.loading && !store.availability" class="text-sm text-ink/50">
              Carregando horários…
            </p>
            <BookingSlotPicker
              v-else-if="store.availability"
              :availability="store.availability"
              :selected-day="store.selectedDay"
              :selected-slot="store.selectedSlot"
              @select-day="store.selectedDay = $event"
              @select-slot="store.selectSlot($event)"
            />
          </section>

          <section v-if="store.selectedSlot" class="mb-7">
            <h2 class="mb-3 font-display text-lg font-semibold">Seus dados</h2>
            <BookingClientForm
              v-model:nome="store.clientName"
              v-model:telefone="store.clientPhone"
              v-model:email="store.clientEmail"
            />
          </section>

          <section v-if="store.selectedSlot && store.selectedService && store.selectedProfessional" class="mb-7">
            <h2 class="mb-3 font-display text-lg font-semibold">Resumo</h2>
            <BookingSummary
              :service="store.selectedService"
              :professional="store.selectedProfessional"
              :day="store.selectedDay ?? ''"
              :slot="store.selectedSlot"
              :client-name="store.clientName"
              :client-phone="store.clientPhone"
            />
          </section>

          <button
            v-if="store.selectedSlot"
            type="button"
            class="w-full rounded-lg bg-accent px-5 py-4 font-display text-base font-semibold text-white transition-colors hover:bg-accent-dark disabled:opacity-50"
            :disabled="!store.canSubmit || store.loading"
            @click="store.submitBooking(slug)"
          >
            {{ store.loading ? 'Processando…' : 'Agendar e pagar sinal' }}
          </button>
          <p v-if="!store.canSubmit && store.selectedSlot" class="mt-2 text-center text-xs text-ink/50">
            Preencha nome e WhatsApp para continuar
          </p>

          <p v-if="store.error" class="mt-4 rounded-lg bg-danger/10 px-4 py-3 text-center text-sm text-danger">
            {{ store.error }}
          </p>
        </template>

        <!-- Pagamento (hold) -->
        <template v-if="store.step === 'hold' && store.hold">
          <h2 class="mb-1 font-display text-xl font-semibold">Pague o sinal para confirmar</h2>
          <p class="mb-4 text-sm text-ink/60">
            Assim que o pagamento cair, seu horário fica garantido e você recebe a confirmação.
          </p>
          <BookingPixPayment
            :hold="store.hold"
            :remaining-seconds="remaining"
            @expired="store.step = 'expired'; pararTimers()"
          />
          <p class="mt-4 text-center text-xs text-ink/50">
            Aguardando confirmação do pagamento…
          </p>
        </template>

        <!-- Confirmado -->
        <template v-if="store.step === 'confirmed'">
          <div class="rounded-lg border border-brand/30 bg-white px-6 py-10 text-center">
            <div class="mx-auto flex h-14 w-14 items-center justify-center rounded-full bg-brand-soft">
              <span class="text-2xl text-brand">&#10003;</span>
            </div>
            <h2 class="mt-4 font-display text-xl font-semibold text-brand">Agendamento confirmado!</h2>
            <p class="mt-2 text-sm text-ink/60">
              Seu horário está garantido. Você receberá os detalhes no WhatsApp.
            </p>
            <div class="mt-6 grid gap-2.5">
              <button
                type="button"
                class="rounded-lg bg-brand px-5 py-3 font-semibold text-white transition-colors hover:bg-brand-dark"
                @click="baixarCalendario"
              >
                Adicionar ao calendário
              </button>
              <a
                v-if="linkWhats"
                :href="linkWhats"
                target="_blank"
                rel="noopener"
                class="rounded-lg border border-brand px-5 py-3 font-semibold text-brand transition-colors hover:bg-brand-soft"
              >
                Falar no WhatsApp
              </a>
            </div>
          </div>
        </template>

        <!-- Expirado -->
        <template v-if="store.step === 'expired'">
          <div class="rounded-lg border border-ink/10 bg-white px-6 py-10 text-center">
            <div class="mx-auto flex h-14 w-14 items-center justify-center rounded-full bg-danger/10">
              <span class="text-2xl text-danger">&#9203;</span>
            </div>
            <h2 class="mt-4 font-display text-xl font-semibold">Tempo esgotado</h2>
            <p class="mt-2 text-sm text-ink/60">
              O horário foi liberado. Escolha outro horário para tentar novamente.
            </p>
            <button
              type="button"
              class="mt-6 rounded-lg bg-brand px-6 py-3 font-semibold text-white transition-colors hover:bg-brand-dark"
              @click="reiniciar"
            >
              Ver horários disponíveis
            </button>
          </div>
        </template>
      </main>

      <footer class="border-t border-ink/10 px-5 py-6 text-center text-xs text-ink/45">
        Agendamento protegido por ReservaZap
      </footer>
    </template>
  </div>
</template>
