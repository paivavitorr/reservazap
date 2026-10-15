# Changelog

## Não lançado

### Adicionado
- Fluxo público de agendamento com slots de horário reais (expediente, duração e ocupação)
- Pagamento de sinal via Pix: mock (dev) e Mercado Pago (produção) com webhook assinado e auditoria em `payments`
- Notificações WhatsApp: confirmação, lembretes 24h/1h e aviso de expiração — Evolution API ou mock
- Painel do prestador com autenticação JWT: CRUD de serviços/profissionais, expediente, agenda, ações de status e resumo
- Cadastro de estabelecimento (tenant + dono + expediente padrão) pelo próprio painel
- LGPD: exportação e anonimização de dados de clientes
- Seed demo (`make be-seed`), CI no GitHub Actions e stack Docker completa

### Corrigido
- Envio de horário em UTC (o formulário enviava horário local sem offset)
- Polling de status fora do contexto do Nuxt e botão de copiar Pix
- Isolamento entre tenants no catálogo, agendamentos e status
- Cobranças Pix órfãs (canceladas quando o agendamento falha) e confirmação idempotente
- Scheduler de expiração com cron válido e execução junto do servidor
