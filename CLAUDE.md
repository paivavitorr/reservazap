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
- **Pagamento:** Pix (provider trait — mock em dev, PSP real em prod)

## Comandos

```bash
make db-up          # Subir PostgreSQL via Docker
make be-run         # Rodar backend (cargo loco start)
make be-migrate     # Rodar migrations
make fe-install     # Instalar deps do frontend
make fe-dev         # Rodar frontend em dev
```

## Convenções

- Código em **português** (nomes de variáveis, comentários, commits)
- Multi-tenant via slug na URL pública (`/{slug}`)
- Agendamentos com hold de 10min via background task (expiração automática)