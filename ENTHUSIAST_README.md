# Enthusiast AI Setup Note

The `enthusiast/` directory contains the Enthusiast AI framework for customer service.

## Setup Instructions

1. Clone Enthusiast (if not already present):
```bash
git clone https://github.com/upsidelab/enthusiast.git
```

2. Copy the environment file:
```bash
cp enthusiast/server/sample.env enthusiast/server/.env
```

3. Edit `enthusiast/server/.env` and configure:
- Database settings (use shared PostgreSQL)
- OpenAI API key
- Admin credentials

4. The docker-compose.yml already includes all Enthusiast services

## Why isn't it in the repo?

The enthusiast directory is large and is available from the official GitHub repo.
For collaboration, team members should clone it separately following the instructions above.

## Documentation

- Setup: `ENTHUSIAST_SETUP_STATUS.md`
- Integration: `ENTHUSIAST_INTEGRATION.md`
- Official docs: https://upsidelab.io/tools/enthusiast/docs
