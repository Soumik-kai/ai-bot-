# tg-ai-bot-rust

Telegram AI assistant (Rust) with real-time search, model fallback, image generation, and group-only enforcement. Deployable to Vercel (serverless webhook).

## Features

- Activation via `/ask` command or `@botusername` mention in configured group.
- Group-only usage; private chat restricted to admin/authorized IDs.
- Real-time web search integrated into prompts.
- LLM pool with prioritized providers and automatic fallback.
- Image generation via multiple providers with rotation.
- Simulated streaming using `typing` action and message edits.
- Multiple API keys support and admin commands to manage keys/users.
- Deployable to Vercel free tier as a serverless webhook.

## Quick start

1. Copy `examples/env.example` to `.env` and fill values.
2. Provision Postgres and Redis (managed providers recommended).
3. Run DB migrations in `sql/schema.sql`.
4. Local dev:
   ```bash
   cargo run --release