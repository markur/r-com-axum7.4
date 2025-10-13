# Enthusiast AI Integration

## Overview
Enthusiast is integrated as a microservice for AI-powered customer service. It runs alongside the Rust backend and shares the PostgreSQL database.

## Architecture

```
┌─────────────────┐      ┌──────────────────┐      ┌─────────────────┐
│   Frontend      │─────▶│  Rust Backend    │─────▶│   PostgreSQL    │
│  (React/Next)   │      │  (Port 3000)     │      │   (Port 5433)   │
└─────────────────┘      └──────────────────┘      └─────────────────┘
         │                        │                          │
         │                        │                          │
         │                        ▼                          │
         │               ┌──────────────────┐               │
         └──────────────▶│ Enthusiast API   │───────────────┘
                         │  (Port 10000)    │
                         └──────────────────┘
                                  │
                         ┌────────┴─────────┐
                         │                  │
                    ┌────▼────┐       ┌────▼────┐
                    │  Worker │       │  Beat   │
                    │ (Celery)│       │(Cron)   │
                    └────┬────┘       └─────────┘
                         │
                    ┌────▼────┐
                    │  Redis  │
                    └─────────┘
```

## Services

### 1. **enthusiast-api** (Port 10000)
- Django REST API for AI conversations
- Handles customer service queries
- RAG (Retrieval-Augmented Generation)
- Vector search for knowledge base

### 2. **enthusiast-worker**
- Celery worker for async processing
- Handles AI response generation
- Processes long-running tasks

### 3. **enthusiast-beat**
- Celery beat scheduler
- Handles scheduled tasks
- Periodic knowledge base updates

### 4. **enthusiast-redis**
- Message broker for Celery
- Task queue management

### 5. **enthusiast-frontend** (Port 10001)
- Admin UI for knowledge base management
- Conversation history viewer
- Agent configuration

## API Endpoints

### Create Conversation
```http
POST http://localhost:10000/api/conversations
Content-Type: application/json

{
  "data_set_id": 1
}
```

### Send Message
```http
POST http://localhost:10000/api/conversations/{conversation_id}
Content-Type: application/json

{
  "question_message": "Where is my order?",
  "data_set_id": 1
}
```

### Check Status
```http
GET http://localhost:10000/api/task_status/{task_id}
```

### Get Conversation
```http
GET http://localhost:10000/api/conversations/{conversation_id}
```

## Configuration

### Environment Variables
Location: `/enthusiast/server/.env`

Key settings:
- `OPENAI_API_KEY` - Required for AI functionality
- `ECL_DB_HOST=db` - Shared PostgreSQL instance
- `ECL_ADMIN_EMAIL` / `ECL_ADMIN_PASSWORD` - Admin credentials

### Database
- Enthusiast uses a separate database: `enthusiast`
- Runs on the same PostgreSQL instance as your main app
- Migrations run automatically on startup

## Integration with Rust Backend

Your Rust backend can proxy requests to Enthusiast:

```rust
// Example: POST /api/chat/ask
// Proxies to Enthusiast API
async fn customer_service_chat(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, StatusCode> {
    // Call Enthusiast API at http://enthusiast-api:10000
    // Handle async task polling
    // Return response to frontend
}
```

## Setup Steps

1. **Add OpenAI API Key**
   ```bash
   # Edit enthusiast/server/.env
   OPENAI_API_KEY=sk-your-key-here
   ```

2. **Access Admin UI**
   - URL: http://localhost:10001
   - Default credentials: admin@example.com / changeme123

3. **Create Knowledge Base**
   - Add data sets (products, policies, FAQs)
   - Upload documents
   - Configure RAG settings

4. **Create Agent**
   - Define agent behavior
   - Set prompts
   - Configure tools

## Use Cases

### Customer Support
- "Where is my order #12345?"
- "How do I return an item?"
- "What's your shipping policy?"

### Product Recommendations
- "I need a gift for my mom who likes wine"
- "What's similar to product X?"
- "Do you have any deals?"

### Order Management
- "Can I change my delivery address?"
- "When will my order arrive?"
- "How do I track my package?"

## Next Steps

1. Add OpenAI API key to enable AI
2. Create knowledge base with your product catalog
3. Build Rust proxy endpoints for `/api/chat/*`
4. Add chat widget to frontend
5. Train agent with common customer queries

## Troubleshooting

### Check service status
```bash
docker compose ps
```

### View logs
```bash
docker compose logs enthusiast-api
docker compose logs enthusiast-worker
```

### Restart services
```bash
docker compose restart enthusiast-api enthusiast-worker enthusiast-beat
```

## Resources

- Enthusiast Docs: https://upsidelab.io/tools/enthusiast/docs
- GitHub: https://github.com/upsidelab/enthusiast
- Admin UI: http://localhost:10001
- API: http://localhost:10000
