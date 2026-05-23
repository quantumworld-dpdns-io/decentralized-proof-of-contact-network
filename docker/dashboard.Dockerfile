# Dashboard Dockerfile (Next.js)
# Stage 1: Build the Next.js application
FROM node:20-alpine AS builder

ARG NEXT_PUBLIC_API_URL
ARG NEXT_PUBLIC_WS_URL

WORKDIR /app

# Install dependencies
COPY dashboard/package*.json dashboard/tsconfig*.json ./
RUN npm ci --ignore-scripts

# Copy source code
COPY dashboard/ .

# Build the application
ENV NEXT_PUBLIC_API_URL=${NEXT_PUBLIC_API_URL:-http://localhost:3000}
ENV NEXT_PUBLIC_WS_URL=${NEXT_PUBLIC_WS_URL:-ws://localhost:9090}

RUN npm run build

# Stage 2: Production server
FROM node:20-alpine AS runner

WORKDIR /app

ENV NODE_ENV=production
ENV PORT=3000

# Create non-root user
RUN addgroup --system --gid 1001 nodejs && \
    adduser --system --uid 1001 nextjs

# Copy built assets from builder
COPY --from=builder /app/public ./public
COPY --from=builder --chown=nextjs:nodejs /app/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/.next/static ./.next/static

USER nextjs

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/ || exit 1

CMD ["node", "server.js"]

LABEL org.opencontainers.image.title="POI Dashboard" \
      org.opencontainers.image.description="Decentralized Proof-of-Contact Network Dashboard" \
      org.opencontainers.image.source="https://github.com/quantumworld-dpdns-io/decentralized-proof-of-contact-network" \
      org.opencontainers.image.licenses="MIT"
