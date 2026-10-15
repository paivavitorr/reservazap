# Arquitetura do ReservaZap

## Visão de domínio

```
Tenant (estabelecimento, slug público)
├── User (dono/funcionário, login JWT)
├── Service (duração, preço, % de sinal)
├── Professional
├── WorkingHours (faixas por weekday — 0=segunda .. 6=domingo)
└── Appointment (hold → confirmed → completed/no_show/cancelled/expired)
    └── Payment (trilha de auditoria: pending → paid/late_paid/cancelled/expired)
```

## Ciclo do agendamento (anti-no-show)

1. **Hold** — o cliente escolhe um slot e um hold de 10min (`booking_hold_minutes`) é criado junto da cobrança Pix.
2. **Confirmação** — webhook do PSP confirma o sinal (`confirmar_por_txid`, idempotente) → `confirmed`.
3. **Expiração** — o scheduler (`expire_holds`, 1min) marca holds vencidos como `expired` e libera o horário.
4. **Lembretes** — `send_reminders` (5min) dispara WhatsApp 24h e 1h antes, uma única vez por agendamento.
5. **Encerramento** — o painel registra `completed`, `no_show` ou `cancelled` (com motivo) por transição atômica.

## Decisões importantes

- **Double-booking impedido no banco**: constraint `EXCLUDE USING gist` (`no_overlap`) em
  `appointments` cobre `professional_id` + intervalo `[starts_at, ends_at)` para status
  `hold/confirmed`. Corridas entre requisições não conseguem furar o horário.
- **Confirmação atômica**: `UPDATE … WHERE status='hold' AND hold_expires_at > now()`; a resposta
  `false` dispara re-leitura para distinguir corrida (outra entrega confirmou) de pagamento tardio.
- **Pagamento tardio**: nunca é descartado em silêncio — vira `late_paid` em `payments` para
  reconciliação manual.
- **Webhook do Mercado Pago**: assinatura `x-signature` validada por HMAC-SHA256 e **situação
  conferida no PSP** antes de agir (o corpo do webhook não é confiável sozinho).
- **Horários**: trafegam em UTC (ISO-8601); expediente e exibição usam `tenants.timezone`
  (`chrono-tz`). Slots são gerados em passos de 30min dentro das faixas do expediente.
- **Cobrança órfã**: se o insert do agendamento falhar, a cobrança Pix criada é cancelada.
- **Multi-tenant**: toda query administrativa escopa por `tenant_id` do usuário do JWT; consultas
  públicas validam `service/professional` pertencem ao slug.

## Providers (selecionáveis por ambiente)

| Trait | Mock (dev) | Produção |
|---|---|---|
| `PixProvider` | `MockPixProvider` (BR Code + QR SVG) | `MercadoPagoPixProvider` |
| `NotificationProvider` | `MockNotificationProvider` (log) | `EvolutionApiProvider` |

## Segurança

- Painel: JWT HS512 (Loco `auth.jwt`), senhas argon2, login sem enumeração de contas.
- Webhook Pix: HMAC-SHA256 + consulta ao PSP; secret obrigatório em produção.
- LGPD: exportação (JSON) e anonimização de dados pessoais pelo painel.
