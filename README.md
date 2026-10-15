# ReservaZap

Agendamento online com **sinal via Pix** para pequenos prestadores de serviço. O cliente escolhe o horário pelo link, paga o sinal na hora e o horário fica garantido — a missão é zerar falta.

## Como funciona

1. O prestador cadastra seu estabelecimento e recebe um link (`/seu-link`).
2. O cliente abre o link, escolhe serviço, profissional e horário livre.
3. O horário é reservado por 10 minutos enquanto o cliente paga o sinal (QR Code Pix).
4. Sinal pago → agendamento confirmado e confirmação no WhatsApp.
5. Lembretes automáticos 24h e 1h antes do atendimento.

## Stack

| Camada | Tecnologia |
|---|---|
| Backend | Rust, [Loco.rs](https://loco.rs), SeaORM, PostgreSQL 16 |
| Frontend | Nuxt 3, Vue 3, Pinia, Tailwind CSS |
| Pagamentos | Pix — mock em dev, Mercado Pago em produção (`PIX_PROVIDER`) |
| WhatsApp | Evolution API em produção (`NOTIFICATION_PROVIDER`), mock em dev |
| Infra | Docker Compose, GitHub Actions |

## Começando em 5 minutos

Pré-requisitos: Docker, Rust (stable) e pnpm.

```bash
make dev          # sobe o PostgreSQL (localhost:5433)
make be-seed      # cria o estabelecimento demo (barbearia-do-zeu)
make be-run       # backend com scheduler em http://localhost:3000
make fe-dev       # frontend em http://localhost:3001
```

- Demo pública: [http://localhost:3001/barbearia-do-zeu](http://localhost:3001/barbearia-do-zeu)
- Painel do prestador: [http://localhost:3001/painel](http://localhost:3001/painel) (crie sua conta em "Cadastre seu estabelecimento")

## Comandos

| Comando | Descrição |
|---|---|
| `make db-up` / `make db-down` | PostgreSQL via Docker (porta 5433) |
| `make be-run` | Backend + scheduler (expiração de holds e lembretes) |
| `make be-migrate` / `make be-seed` | Migrations / dados de demonstração |
| `make be-test` / `make be-lint` | Testes / fmt + clippy do backend |
| `make fe-dev` / `make fe-typecheck` / `make fe-test` | Frontend |
| `make test` / `make lint` | Tudo |

## Estrutura

```
reservazap/
├── backend/        # API Loco.rs (controllers → services → models)
├── frontend/       # Nuxt 3 (páginas públicas + /painel)
├── migration/      # Migrations SeaORM
└── docker-compose.yml
```

## Variáveis de ambiente

Copie `.env.example` e ajuste. As principais:

| Variável | Padrão | Descrição |
|---|---|---|
| `DATABASE_URL` | `postgres://…localhost:5433/reservazap` | Banco PostgreSQL |
| `JWT_SECRET` | — | Segredo base64 (≥ 32 bytes) do painel — `openssl rand -base64 48` |
| `PIX_PROVIDER` | `mock` | `mock` \| `mercadopago` |
| `MERCADOPAGO_ACCESS_TOKEN` | — | Token de acesso do Mercado Pago |
| `MERCADOPAGO_WEBHOOK_SECRET` | — | Segredo de assinatura do webhook (obrigatório em produção) |
| `NOTIFICATION_PROVIDER` | `mock` | `mock` \| `evolution` |
| `EVOLUTION_API_URL` / `EVOLUTION_API_KEY` / `EVOLUTION_INSTANCE` | — | Evolution API |
| `NUXT_API_BASE_INTERNAL` | `http://127.0.0.1:3000/api` | Backend alcançado no SSR |

## Contratos da API (resumo)

**Públicas** (`/api/public/{slug}/...`): `GET catalog`, `GET availability`, `POST bookings/intent`, `GET bookings/{id}/status`.

**Webhooks** (`/api/webhooks/...`): `POST pix` (mock), `POST pix/mercadopago` (assinado via `x-signature`).

**Painel** (`/api/auth`, `/api/painel` — Bearer JWT): registro/login, CRUD de serviços/profissionais, expediente, agenda, ações de status, resumo e LGPD (`clientes/dados`, `clientes/anonimizar`).

## Produção

```bash
# gera o segredo do JWT e preencha o .env
openssl rand -base64 48

# stack completa (postgres + backend + frontend)
docker compose --profile full up --build
```

- Backend: `http://localhost:3000` (healthcheck em `/_ping`)
- Frontend: `http://localhost:3001`
- Configure o webhook do Mercado Pago para `https://seu-dominio/api/webhooks/pix/mercadopago`
- O scheduler roda junto do backend (`start --scheduler`): expira holds a cada 1min e envia lembretes a cada 5min

## Testes

```bash
make be-test   # unit + integração (requer Postgres via make db-up)
make fe-test   # vitest
make lint      # fmt/clippy + typecheck
```

A CI (`.github/workflows/ci.yml`) roda fmt, clippy, testes do backend com Postgres e typecheck/testes/build do frontend em cada push.

## Privacidade (LGPD)

O painel permite exportar (JSON) e anonimizar os dados pessoais de um cliente pelo WhatsApp. Dados trafegam em UTC e são exibidos no fuso do estabelecimento.
