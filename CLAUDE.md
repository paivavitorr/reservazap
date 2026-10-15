# ReservaZap — Contexto do Projeto

## Visão Geral

Micro-SaaS de agendamentos automatizados com cobrança de sinal via Pix para pequenos prestadores de serviço. Objetivo: zerar o no-show.

## Estrutura

```
reservazap/
├── backend/       # Rust + Loco.rs + SeaORM + PostgreSQL
├── frontend/      # Nuxt 3 + Vue 3 + Pinia + Tailwind CSS
├── docs/          # Documentação técnica
├── docker-compose.yml
├── Makefile
└── .env.example
```

## Stack

- **Backend:** Rust, Loco.rs (axum), SeaORM, PostgreSQL 16, Tokio
- **Frontend:** Nuxt 3, Vue 3 (Composition API), Pinia, Tailwind CSS v4
- **Infra:** Docker Compose
- **Pagamento:** Pix (provider trait — mock em dev, Mercado Pago em prod via `PIX_PROVIDER`)
- **Notificações:** WhatsApp (provider trait — Evolution API via `NOTIFICATION_PROVIDER`, mock em dev)

## Comandos

```bash
make db-up          # Subir PostgreSQL via Docker (localhost:5433)
make be-run         # Rodar backend com scheduler (cargo loco start --scheduler)
make be-migrate     # Rodar migrations
make be-seed        # Popular demo (barbearia-do-zeu)
make be-test        # Testes do backend (requer Postgres ativo)
make be-lint        # fmt + clippy
make fe-install     # Instalar deps do frontend
make fe-dev         # Rodar frontend em dev (localhost:3001)
make fe-typecheck   # Typecheck do frontend
make fe-test        # Testes do frontend
make dev            # Sobe o Postgres e mostra os próximos passos
```

## Convenções

- Código em **português** (nomes de variáveis, comentários, commits)
- Multi-tenant via slug na URL pública (`/{slug}`)
- Agendamentos com hold de 10min via background task (expiração automática)
- Backend em `localhost:3000` (`/api/...`); frontend em `localhost:3001` com proxy `/api` (Nuxt dev) e CORS habilitado no backend
- Horários trafegam em UTC (ISO-8601); exibição no fuso do tenant (`tenants.timezone`)
- Dados de demo: slug `barbearia-do-zeu` criado por `make be-seed`
- Painel do prestador em `/painel` (JWT); documentação principal no `README.md` e `docs/ARQUITETURA.md`