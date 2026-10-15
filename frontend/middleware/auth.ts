export default defineNuxtRouteMiddleware((to) => {
  if (to.path === '/painel/login' || to.path === '/painel/registro') return
  if (!tokenAtual()) {
    return navigateTo('/painel/login')
  }
})
