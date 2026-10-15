# build com pnpm + Nuxt standalone (Nitro)
FROM node:22-alpine AS build
WORKDIR /app

RUN corepack enable && corepack prepare pnpm@11 --activate

COPY frontend/package.json frontend/pnpm-lock.yaml frontend/pnpm-workspace.yaml ./
RUN pnpm install --frozen-lockfile

COPY frontend .

# o destino do proxy /api é fixado no build do Nitro
ARG NUXT_API_PROXY_TARGET=http://127.0.0.1:3000
ENV NUXT_API_PROXY_TARGET=$NUXT_API_PROXY_TARGET
RUN pnpm build

FROM node:22-alpine
WORKDIR /app
ENV NODE_ENV=production
ENV PORT=3001
COPY --from=build /app/.output ./.output

EXPOSE 3001
CMD ["node", ".output/server/index.mjs"]
