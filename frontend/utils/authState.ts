import type { UsuarioPainel } from '~/types/painel'

/** Estado de autenticação fora do Pinia/payload — seguro para uso em handlers. */
export const authToken = ref<string | null>(null)
export const authUser = ref<UsuarioPainel | null>(null)

export function lerToken(): string | null {
  if (import.meta.server) return null
  const match = document.cookie.match(/(?:^|;\s*)rz_token=([^;]*)/)
  return match ? decodeURIComponent(match[1]) : null
}

export function gravarToken(token: string | null) {
  if (import.meta.server) return
  if (token) {
    document.cookie = `rz_token=${encodeURIComponent(token)}; path=/; max-age=${60 * 60 * 24 * 7}`
  } else {
    document.cookie = 'rz_token=; path=/; max-age=0'
  }
  authToken.value = token
}

export function tokenAtual(): string | null {
  return authToken.value ?? lerToken()
}
