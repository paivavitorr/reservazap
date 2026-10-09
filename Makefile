.PHONY: db-up db-down be-run be-check be-migrate fe-install fe-dev

db-up:
	docker compose up -d

db-down:
	docker compose down

be-run:
	cd backend && cargo loco start

be-check:
	cd backend && cargo check

be-migrate:
	cd backend && cargo loco db migrate

fe-install:
	cd frontend && pnpm install

fe-dev:
	cd frontend && pnpm dev

dev: db-up
	@echo "PostgreSQL disponível em localhost:5432"