import { defineStore } from 'pinia'
import type { AuthResponse, UsuarioPainel } from '~/types/painel'

type ApiClient = ReturnType<typeof $fetch.create>

/** Fora do state do Pinia — não serializa no payload SSR. */
let api: ApiClient | null = null
/** Evita chamadas concorrentes de /auth/eu (layout + página). */
let promessaRestaurar: Promise<void> | null = null

interface AuthState {
  user: UsuarioPainel | null
  carregado: boolean
}

export const useAuthStore = defineStore('auth', {
  state: (): AuthState => ({
    user: null,
    carregado: false,
  }),

  getters: {
    autenticado: (state) => !!state.user,
  },

  actions: {
    init(client: ApiClient) {
      api = client
    },

    /** Restaura a sessão a partir do cookie (chamar no setup do painel). */
    restaurar() {
      if (!api || this.carregado || promessaRestaurar) {
        return promessaRestaurar ?? Promise.resolve()
      }
      promessaRestaurar = (async () => {
        if (!tokenAtual()) return
        try {
          this.user = await api!<UsuarioPainel>('/auth/eu')
          authUser.value = this.user
        } catch {
          gravarToken(null)
          this.user = null
        } finally {
          this.carregado = true
        }
      })()
      return promessaRestaurar
    },

    async login(email: string, password: string) {
      if (!api) throw new Error('api não inicializada')
      const data = await api!<AuthResponse>('/auth/login', {
        method: 'POST',
        body: { email, password },
      })
      gravarToken(data.token)
      this.user = data.user
      authUser.value = data.user
      this.carregado = true
    },

    async registrar(payload: {
      tenant_name: string
      slug: string
      phone: string
      name: string
      email: string
      password: string
    }) {
      if (!api) throw new Error('api não inicializada')
      const data = await api!<AuthResponse>('/auth/registro', {
        method: 'POST',
        body: payload,
      })
      gravarToken(data.token)
      this.user = data.user
      authUser.value = data.user
      this.carregado = true
    },

    logout() {
      gravarToken(null)
      this.user = null
      authUser.value = null
      this.carregado = true
    },
  },
})
