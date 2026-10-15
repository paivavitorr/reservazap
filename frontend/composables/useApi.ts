/**
 * Cliente HTTP com `baseURL` fixada no momento da chamada.
 *
 * - Cliente: `/api` (proxy do Nitro — sem CORS).
 * - SSR: URL interna do backend (`apiBaseInternal`).
 *
 * Anexa `Authorization: Bearer` quando há token de painel. Chamar `useApi()`
 * dentro de `setup()`; a instância devolvida pode ser usada depois em
 * timers/handlers — `useRuntimeConfig()` fora do contexto do Nuxt dispara
 * "nuxt instance unavailable".
 */
export function useApi() {
  const config = useRuntimeConfig()
  const baseURL = import.meta.server
    ? config.apiBaseInternal || config.public.apiBase
    : config.public.apiBase
  return $fetch.create({
    baseURL,
    onRequest({ options }) {
      const token = tokenAtual()
      if (token) {
        options.headers.set('Authorization', `Bearer ${token}`)
      }
    },
  })
}
