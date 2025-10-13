# Enthusiast AI Setup - Current Status

## ✅ Completed

1. **Cloned Enthusiast Repository**
   - Location: `/home/markur/R-Com/enthusiast/`
   - Source: https://github.com/upsidelab/enthusiast

2. **Created Environment Configuration**
   - File: `/enthusiast/server/.env`
   - Configured to use shared PostgreSQL instance
   - Separate database: `enthusiast`
   - Redis broker configured

3. **Integrated with Docker Compose**
   - Added 5 new services to `docker-compose.yml`:
     - `enthusiast-redis` - Message broker
     - `enthusiast-api` - Django REST API (Port 10000)
     - `enthusiast-worker` - Celery async worker
     - `enthusiast-beat` - Celery scheduler
     - `enthusiast-frontend` - Admin UI (Port 10001)

4. **Started Build Process**
   - Building Python services (Django backend)
   - Building Node services (React frontend)
   - Pulling Redis image (complete)

## ⏳ In Progress

- Building Docker images for all services
- This will take 5-10 minutes (downloading dependencies)

## 📋 Next Steps

### 1. Wait for Build to Complete
Check status:
```bash
docker compose ps
```

### 2. Add OpenAI API Key (Required)
Edit `/enthusiast/server/.env` and add:
```env
OPENAI_API_KEY=sk-your-openai-key-here
```

Then restart:
```bash
docker compose restart enthusiast-api enthusiast-worker
```

### 3. Access Admin UI
- URL: http://localhost:10001
- Login: admin@example.com / changeme123

### 4. Create Knowledge Base
In the admin UI:
1. Create a "Data Set" (e.g., "Customer Service KB")
2. Add documents:
   - Shipping policies
   - Return policies
   - Product information
   - FAQs

### 5. Create AI Agent
Configure:
- Agent type: Customer Service
- Behavior: Helpful, friendly, knowledgeable
- Tools: RAG search, product lookup

### 6. Test API
```bash
# Create conversation
curl -X POST http://localhost:10000/api/conversations \
  -H "Content-Type: application/json" \
  -d '{"data_set_id": 1}'

# Send message
curl -X POST http://localhost:10000/api/conversations/1 \
  -H "Content-Type: application/json" \
  -d '{
    "question_message": "Where is my order?",
    "data_set_id": 1
  }'
```

### 7. Build Rust Integration
Create proxy endpoints in your Rust backend:
- `POST /api/chat/start` - Create conversation
- `POST /api/chat/message` - Send message
- `GET /api/chat/:id` - Get conversation history

### 8. Add Chat Widget to Frontend
Integrate with your React frontend to show customer service chat.

## 🔑 Required API Keys

### OpenAI (Required)
- Get key at: https://platform.openai.com/api-keys
- Models supported: GPT-4, GPT-3.5-turbo
- Cost: ~$0.002 per 1K tokens (GPT-3.5)

### Alternative: Self-Hosted LLMs (Optional)
If you don't want to use OpenAI, you can use:
- Mistral
- LLaMA
- Deepseek

But OpenAI is recommended to start.

## 📊 Service Ports

- `3000` - Your Rust backend
- `5433` - PostgreSQL (host port)
- `10000` - Enthusiast API
- `10001` - Enthusiast Admin UI

## 🐛 Troubleshooting

### Check build progress
```bash
docker compose logs enthusiast-api --follow
```

### View all services
```bash
docker compose ps
```

### Restart if needed
```bash
docker compose restart enthusiast-api enthusiast-worker enthusiast-beat
```

### Check database
```bash
docker compose exec db psql -U postgres -d enthusiast -c "\\dt"
```

## 📚 Documentation

- Full integration guide: `ENTHUSIAST_INTEGRATION.md`
- Enthusiast docs: https://upsidelab.io/tools/enthusiast/docs
- API reference: https://upsidelab.io/tools/enthusiast/docs/integrate/conversation-via-api

## 💡 Use Cases

Once setup is complete, customers can ask:
- "Where is my order #12345?"
- "How do I return an item?"
- "What's your shipping policy?"
- "I need a gift for someone who likes wine"
- "Do you have any deals?"
- "Can I change my delivery address?"

The AI will search your knowledge base and provide accurate, helpful responses!
