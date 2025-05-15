# Rust E-commerce Platform V1

## Overview
A minimal, secure, and scalable e-commerce platform built with Rust for the backend (Axum), and two frontend options: Yew (Rust/WASM) and React (JS/TS). The backend uses PostgreSQL for data storage and integrates with Stripe for payments.

---

## Tech Stack
- **Backend:** Rust (Axum, sqlx, serde, argon2, jsonwebtoken, stripe)
- **Database:** PostgreSQL (via Docker Compose)
- **Frontend:**
  - **Yew** (Rust → WASM, deployable to Vercel)
  - **React** (JS/TS, deployable to Vercel)
- **Dev Environment:** Docker Compose for backend + Postgres

---

## Directory Structure
```
/ecommerce-platform
  /backend      # Rust Axum API
  /frontend-yew # Yew (Rust WASM)
  /frontend-react # React (JS/TS)
  docker-compose.yml
  README.md
```

---

## Setup Plan
1. **Backend:**
   - Scaffold Rust project with Axum, sqlx, Stripe, JWT/Argon2
   - Docker Compose for backend + Postgres
2. **Frontend:**
   - Scaffold Yew project (Rust/WASM)
   - Scaffold React project (JS/TS)
3. **README:**
   - Setup and usage instructions for each component

---

## Deployment
- **Frontend:** Deploy either (or both) frontends to Vercel
- **Backend:** Deploy to a Rust-friendly host (Railway, Fly.io, Render, or self-hosted)

---

## Next Steps
- Scaffold backend and both frontends
- Add initial Docker Compose setup
- Provide setup instructions

---

## Rust Installation Verification
- **Verify Rust is installed:**
  - Run:
    ```
    cargo --version
    rustc --version
    ```
  - You should see version numbers for both.