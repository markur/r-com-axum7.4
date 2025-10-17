# Multi-stage Dockerfile for Leptos WASM frontend

# Stage 1: Build the WASM application
FROM rustlang/rust:nightly-bookworm as builder

# Install trunk and wasm32 target
RUN cargo install trunk && \
    rustup target add wasm32-unknown-unknown

# Set working directory
WORKDIR /app

# Copy dependency files first for caching
COPY Cargo.toml ./
COPY Leptos.toml ./

# Copy source code
COPY src ./src
COPY style ./style
COPY index.html ./
COPY public ./public

# Build the WASM app with trunk
RUN trunk build --release

# Stage 2: Serve with nginx
FROM nginx:alpine

# Copy built WASM files from builder
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy custom nginx config for SPA routing
RUN echo 'server { \
    listen 80; \
    server_name _; \
    root /usr/share/nginx/html; \
    index index.html; \
    location / { \
        try_files $uri $uri/ /index.html; \
    } \
    location /api/ { \
        proxy_pass http://backend:3000; \
        proxy_http_version 1.1; \
        proxy_set_header Upgrade $http_upgrade; \
        proxy_set_header Connection "upgrade"; \
        proxy_set_header Host $host; \
        proxy_set_header X-Real-IP $remote_addr; \
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for; \
        proxy_set_header X-Forwarded-Proto $scheme; \
    } \
}' > /etc/nginx/conf.d/default.conf

# Expose port 80
EXPOSE 80

# Start nginx
CMD ["nginx", "-g", "daemon off;"]
