<script setup lang="ts">
const route = useRoute()
const slug = route.params.slug as string

const store = useBookingStore()

onMounted(() => {
  store.fetchCatalog(slug)
})

let statusInterval: ReturnType<typeof setInterval> | null = null

watch(
  () => store.step,
  (step) => {
    if (step === 'hold') {
      statusInterval = setInterval(() => store.checkStatus(slug), 3000)
    } else if (statusInterval) {
      clearInterval(statusInterval)
      statusInterval = null
    }
  }
)

onUnmounted(() => {
  if (statusInterval) clearInterval(statusInterval)
})

function formatCents(cents: number): string {
  return (cents / 100).toLocaleString('pt-BR', { style: 'currency', currency: 'BRL' })
}

function formatMinutes(min: number): string {
  return `${min} min`
}
</script>

<template>
  <div class="min-h-screen bg-gray-50">
    <!-- Loading -->
    <div v-if="store.loading && !store.catalog" class="flex items-center justify-center min-h-screen">
      <p class="text-gray-500">Carregando...</p>
    </div>

    <!-- Erro -->
    <div v-else-if="store.error && !store.catalog" class="flex items-center justify-center min-h-screen">
      <p class="text-red-500">{{ store.error }}</p>
    </div>

    <!-- Catálogo -->
    <div v-else-if="store.catalog">
      <header class="bg-white shadow-sm">
        <div class="max-w-2xl mx-auto px-4 py-6">
          <h1 class="text-2xl font-bold text-gray-900">{{ store.catalog.tenant.name }}</h1>
          <p class="text-sm text-gray-500">Agende seu horário</p>
        </div>
      </header>

      <main class="max-w-2xl mx-auto px-4 py-8">
        <!-- Etapa: catálogo / seleção -->
        <template v-if="store.step === 'catalog' || store.step === 'selecting'">
          <!-- Serviços -->
          <section class="mb-8">
            <h2 class="text-lg font-semibold mb-4">Serviços</h2>
            <div class="grid gap-3">
              <button
                v-for="service in store.catalog.services"
                :key="service.id"
                class="text-left p-4 bg-white rounded-lg border-2 transition-colors"
                :class="store.selectedService?.id === service.id ? 'border-blue-500' : 'border-gray-200 hover:border-gray-300'"
                @click="store.selectService(service)"
              >
                <div class="flex justify-between items-start">
                  <div>
                    <p class="font-medium">{{ service.name }}</p>
                    <p v-if="service.description" class="text-sm text-gray-500 mt-1">{{ service.description }}</p>
                    <p class="text-xs text-gray-400 mt-1">{{ formatMinutes(service.duration_minutes) }}</p>
                  </div>
                  <div class="text-right">
                    <p class="font-semibold">{{ formatCents(service.price_in_cents) }}</p>
                    <p class="text-xs text-gray-400">Sinal: {{ service.signal_percent }}%</p>
                  </div>
                </div>
              </button>
            </div>
          </section>

          <!-- Profissionais -->
          <section v-if="store.selectedService" class="mb-8">
            <h2 class="text-lg font-semibold mb-4">Profissional</h2>
            <div class="grid gap-3">
              <button
                v-for="prof in store.catalog.professionals"
                :key="prof.id"
                class="text-left p-4 bg-white rounded-lg border-2 transition-colors"
                :class="store.selectedProfessional?.id === prof.id ? 'border-blue-500' : 'border-gray-200 hover:border-gray-300'"
                @click="store.selectedProfessional = prof"
              >
                <p class="font-medium">{{ prof.name }}</p>
              </button>
            </div>
          </section>

          <!-- Horário (simplificado - input manual) -->
          <section v-if="store.selectedProfessional" class="mb-8">
            <h2 class="text-lg font-semibold mb-4">Horário</h2>
            <input
              v-model="store.selectedTime"
              type="datetime-local"
              class="w-full p-3 border rounded-lg"
            />
          </section>

          <!-- Dados do cliente -->
          <section v-if="store.selectedTime" class="mb-8">
            <h2 class="text-lg font-semibold mb-4">Seus dados</h2>
            <div class="grid gap-3">
              <input v-model="store.clientName" type="text" placeholder="Nome completo" class="p-3 border rounded-lg" />
              <input v-model="store.clientPhone" type="tel" placeholder="WhatsApp" class="p-3 border rounded-lg" />
              <input v-model="store.clientEmail" type="email" placeholder="E-mail (opcional)" class="p-3 border rounded-lg" />
            </div>
          </section>

          <!-- Botão agendar -->
          <button
            v-if="store.canSubmit"
            :disabled="store.loading"
            class="w-full py-3 bg-green-600 text-white font-semibold rounded-lg hover:bg-green-700 disabled:opacity-50"
            @click="store.submitBooking(slug)"
          >
            {{ store.loading ? 'Processando...' : 'Agendar e pagar sinal' }}
          </button>

          <p v-if="store.error" class="text-red-500 text-sm mt-4 text-center">{{ store.error }}</p>
        </template>

        <!-- Etapa: hold (Pix pendente) -->
        <template v-if="store.step === 'hold' && store.hold">
          <div class="bg-white rounded-lg p-6 shadow-sm text-center">
            <h2 class="text-xl font-semibold mb-2">Pague o sinal para confirmar</h2>
            <p class="text-gray-500 mb-6">Você tem 10 minutos para efetuar o pagamento.</p>

            <div class="bg-gray-50 rounded-lg p-4 mb-4">
              <p class="text-sm text-gray-500 mb-1">Valor do sinal</p>
              <p class="text-3xl font-bold text-green-600">{{ store.signalAmountFormatted }}</p>
            </div>

            <div class="bg-gray-50 rounded-lg p-4 mb-6">
              <p class="text-sm text-gray-500 mb-2">Pix copia e cola</p>
              <div class="flex items-center gap-2">
                <input
                  :value="store.hold.signal.pix.copia_e_cola"
                  readonly
                  class="flex-1 p-2 bg-white border rounded text-xs font-mono"
                />
                <button
                  class="px-4 py-2 bg-blue-600 text-white text-sm rounded hover:bg-blue-700"
                  @click="navigator.clipboard.writeText(store.hold!.signal.pix.copia_e_cola)"
                >
                  Copiar
                </button>
              </div>
            </div>

            <p class="text-sm text-gray-400">Aguardando confirmação do pagamento...</p>
          </div>
        </template>

        <!-- Etapa: confirmado -->
        <template v-if="store.step === 'confirmed'">
          <div class="bg-white rounded-lg p-6 shadow-sm text-center">
            <div class="text-5xl mb-4">&#10003;</div>
            <h2 class="text-xl font-semibold mb-2 text-green-600">Agendamento confirmado!</h2>
            <p class="text-gray-500">Você receberá a confirmação no WhatsApp.</p>
          </div>
        </template>

        <!-- Etapa: expirado -->
        <template v-if="store.step === 'expired'">
          <div class="bg-white rounded-lg p-6 shadow-sm text-center">
            <div class="text-5xl mb-4">&#9203;</div>
            <h2 class="text-xl font-semibold mb-2 text-red-600">Tempo esgotado</h2>
            <p class="text-gray-500 mb-4">O horário foi liberado. Tente novamente.</p>
            <button
              class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
              @click="store.reset(); store.fetchCatalog(slug)"
            >
              Voltar ao início
            </button>
          </div>
        </template>
      </main>
    </div>
  </div>
</template>