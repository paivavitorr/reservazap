.PHONY: db-up db-down be-run be-check be-test be-lint be-migrate be-seed fe-install fe-dev fe-typecheck fe-test dev lint test

db-up:
	docker compose up -d

db-down:
	docker compose down

be-run:
	cd backend && cargo loco start --scheduler

be-check:
	cd backend && cargo check

be-test:
	cd backend && cargo test

be-lint:
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets -- -D warnings

be-migrate:
	cd backend && cargo loco db migrate

be-seed:
	cd backend && cargo loco db seed

fe-install:
	cd frontend && pnpm install

fe-dev:
	cd frontend && pnpm dev

fe-typecheck:
	cd frontend && pnpm typecheck

fe-test:
	cd frontend && pnpm test

lint: be-lint fe-typecheck

test: be-test fe-test

dev: db-up
	@echo "PostgreSQL em localhost:5433 (banco reservazap)"
	@echo "Backend:  make be-run   (http://localhost:3000)"
	@echo "Seed:     make be-seed  (demo em /barbearia-do-zeu)"
	@echo "Frontend: make fe-dev   (http://localhost:3001)"
