# Payment Webhooks Setup Guide

This guide explains how to set up and test payment webhooks for both Stripe and Square payment providers.

## Overview

Payment webhooks enable your application to automatically process orders when payments are completed. The system listens for webhook events from Stripe and Square, verifies their authenticity, creates order records, and can trigger follow-up actions like confirmation emails.

## Architecture

### Webhook Endpoints

- **Stripe**: `POST /api/webhooks/stripe`
- **Square**: `POST /api/webhooks/square`

### Database Tables

The webhook system creates three new tables:

1. **webhook_events** - Logs all incoming webhook events for auditing and idempotency
2. **orders** - Stores completed orders with payment information
3. **order_items** - Stores individual line items for each order

### Flow

1. Payment provider sends webhook event to your endpoint
2. System verifies webhook signature (security)
3. Checks if event was already processed (idempotency)
4. Logs event to `webhook_events` table
5. Processes event based on type:
   - Stripe: `payment_intent.succeeded`, `charge.succeeded`, `checkout.session.completed`
   - Square: `payment.updated` with status `COMPLETED`
6. Creates order record in `orders` table
7. Marks webhook as processed
8. Returns 200 OK to payment provider

## Setup Instructions

### 1. Configure Environment Variables

Add the following to your `.env.docker` file:

```bash
# Stripe Webhook Secret (get from Stripe Dashboard)
STRIPE_WEBHOOK_SECRET=whsec_your_webhook_secret_here

# Square Webhook Configuration (get from Square Developer Dashboard)
SQUARE_WEBHOOK_SIGNATURE_KEY=your_webhook_signature_key_here
SQUARE_WEBHOOK_URL=https://your-domain.com/api/webhooks/square
```

### 2. Run Database Migrations

The migration `20230515000000_create_webhooks_and_orders.sql` creates the necessary tables. If using sqlx, run:

```bash
sqlx migrate run
```

Or rebuild your Docker container which will run migrations automatically.

### 3. Configure Stripe Webhooks

#### Option A: Local Testing with Stripe CLI (Recommended for Development)

1. Install Stripe CLI:
   ```bash
   # macOS
   brew install stripe/stripe-cli/stripe

   # Linux
   wget https://github.com/stripe/stripe-cli/releases/download/v1.17.0/stripe_1.17.0_linux_x86_64.tar.gz
   tar -xzf stripe_1.17.0_linux_x86_64.tar.gz
   ```

2. Login to Stripe:
   ```bash
   stripe login
   ```

3. Forward webhooks to your local server:
   ```bash
   stripe listen --forward-to localhost:3000/api/webhooks/stripe
   ```

4. The CLI will display a webhook signing secret (starts with `whsec_`). Add this to your `.env.docker`:
   ```bash
   STRIPE_WEBHOOK_SECRET=whsec_xxxxx
   ```

5. Test with sample events:
   ```bash
   # Trigger a payment intent succeeded event
   stripe trigger payment_intent.succeeded

   # Trigger a checkout session completed event
   stripe trigger checkout.session.completed
   ```

#### Option B: Production Setup (Stripe Dashboard)

1. Go to [Stripe Dashboard > Developers > Webhooks](https://dashboard.stripe.com/webhooks)
2. Click "Add endpoint"
3. Enter your webhook URL: `https://your-domain.com/api/webhooks/stripe`
4. Select events to listen for:
   - `payment_intent.succeeded`
   - `charge.succeeded`
   - `checkout.session.completed`
5. Click "Add endpoint"
6. Copy the "Signing secret" (starts with `whsec_`)
7. Add to your `.env.docker`:
   ```bash
   STRIPE_WEBHOOK_SECRET=whsec_xxxxx
   ```

### 4. Configure Square Webhooks

#### Option A: Local Testing with ngrok (Recommended for Development)

Square requires HTTPS endpoints, so use ngrok to expose your local server:

1. Install ngrok:
   ```bash
   # macOS
   brew install ngrok

   # Or download from https://ngrok.com/download
   ```

2. Start ngrok tunnel:
   ```bash
   ngrok http 3000
   ```

3. Copy the HTTPS URL (e.g., `https://abc123.ngrok.io`)

4. Go to [Square Developer Dashboard > Applications](https://developer.squareup.com/apps)
5. Select your application
6. Go to "Webhooks" section
7. Click "Add subscription"
8. Enter your webhook URL: `https://abc123.ngrok.io/api/webhooks/square`
9. Select events:
   - `payment.created`
   - `payment.updated`
10. Save and copy the "Signature Key"
11. Update your `.env.docker`:
    ```bash
    SQUARE_WEBHOOK_SIGNATURE_KEY=your_signature_key_here
    SQUARE_WEBHOOK_URL=https://abc123.ngrok.io/api/webhooks/square
    ```

#### Option B: Production Setup (Square Developer Dashboard)

1. Go to [Square Developer Dashboard > Applications](https://developer.squareup.com/apps)
2. Select your application
3. Go to "Webhooks" section
4. Click "Add subscription"
5. Enter your webhook URL: `https://your-domain.com/api/webhooks/square`
6. Select events:
   - `payment.created`
   - `payment.updated`
7. Save and copy the "Signature Key"
8. Update your `.env.docker`:
   ```bash
   SQUARE_WEBHOOK_SIGNATURE_KEY=your_signature_key_here
   SQUARE_WEBHOOK_URL=https://your-domain.com/api/webhooks/square
   ```

### 5. Restart Backend

After configuring environment variables, restart the backend:

```bash
docker compose restart backend
```

## Testing

### Test Stripe Webhooks

Using Stripe CLI:

```bash
# Test payment intent succeeded
stripe trigger payment_intent.succeeded

# Test charge succeeded
stripe trigger charge.succeeded

# Test checkout session completed
stripe trigger checkout.session.completed
```

Check your backend logs:
```bash
docker compose logs -f backend
```

You should see:
```
Payment succeeded! PaymentIntent ID: pi_xxxxx, Amount: 1099 usd
Created order with ID: 123e4567-e89b-12d3-a456-426614174000
```

### Test Square Webhooks

1. Make a test payment using your Square sandbox:
   - Use Square Web Payments SDK to create a payment
   - Or use Square Dashboard to create a test payment

2. Check your backend logs:
   ```bash
   docker compose logs -f backend
   ```

3. You should see:
   ```
   Payment updated! Payment ID: xxxxx, Status: COMPLETED, Amount: 1099 USD
   Created order with ID: 123e4567-e89b-12d3-a456-426614174000
   ```

### Verify Database Records

Connect to your database and check the tables:

```sql
-- Check webhook events
SELECT * FROM webhook_events ORDER BY created_at DESC LIMIT 10;

-- Check orders
SELECT * FROM orders ORDER BY created_at DESC LIMIT 10;

-- Check if webhook was processed
SELECT processed, error_message FROM webhook_events WHERE event_id = 'evt_xxxxx';
```

## Troubleshooting

### Stripe Webhook Signature Verification Failed

**Problem**: Backend logs show "Stripe webhook signature verification failed"

**Solutions**:
1. Verify `STRIPE_WEBHOOK_SECRET` matches the secret from Stripe CLI or Dashboard
2. Ensure webhook secret starts with `whsec_`
3. If using Stripe CLI, restart `stripe listen` and copy the new secret
4. Restart backend after updating environment variable

### Square Webhook Signature Verification Failed

**Problem**: Backend returns 401 Unauthorized

**Solutions**:
1. Verify `SQUARE_WEBHOOK_SIGNATURE_KEY` is correct
2. Verify `SQUARE_WEBHOOK_URL` exactly matches the URL configured in Square Dashboard
3. Ensure you're using HTTPS URL (ngrok or production domain)
4. Check that ngrok tunnel is still active (ngrok URLs change on restart)

### Duplicate Events

**Problem**: Same event processed multiple times

**Solution**: The system includes idempotency checking. If you see duplicates, check:
1. Database has unique constraint on `webhook_events.event_id`
2. `is_event_processed()` function is working correctly

### Database Connection Errors

**Problem**: "Failed to log webhook event: database error"

**Solutions**:
1. Verify database is running: `docker compose ps db`
2. Check migrations ran successfully
3. Verify `DATABASE_URL` in `.env.docker` is correct
4. Check database connection pool size

### No Order Created

**Problem**: Webhook received but no order in database

**Solutions**:
1. Check backend logs for error messages
2. Verify event type is one we handle (see Architecture section)
3. For Square: verify payment status is `COMPLETED`
4. Check `webhook_events` table for `error_message` column

## Security Considerations

1. **Always verify webhook signatures** - The system verifies all incoming webhooks using HMAC signatures
2. **Use HTTPS in production** - Never expose webhook endpoints over HTTP
3. **Keep secrets secure** - Never commit `.env.docker` to git (already in .gitignore)
4. **Monitor for failed webhooks** - Set up alerts for `processed = false` in `webhook_events` table
5. **Implement rate limiting** - Consider adding rate limiting to webhook endpoints in production

## Next Steps

After webhooks are working:

1. **Integrate email confirmations** - Update `send_order_confirmation_email()` in webhook handlers to use the existing `lettre_email` module
2. **Add order items** - Parse line items from payment metadata and insert into `order_items` table
3. **Update inventory** - Decrement product inventory when orders are created
4. **Cart abandonment** - Track payment intents that never complete (no webhook received)
5. **Admin notifications** - Send SMS/email to admin when large orders are placed

## API Reference

### Webhook Event Response

Both endpoints return:

```json
{
  "received": true
}
```

Or for duplicate events:

```json
{
  "received": true,
  "duplicate": true
}
```

### Database Schema

#### webhook_events

```sql
id UUID PRIMARY KEY
provider VARCHAR(50)         -- 'stripe' or 'square'
event_type VARCHAR(100)      -- e.g. 'payment_intent.succeeded'
event_id VARCHAR(255) UNIQUE -- Provider's event ID
payload JSONB                -- Full event payload
processed BOOLEAN            -- Whether event was processed
processed_at TIMESTAMP       -- When it was processed
error_message TEXT           -- Any processing errors
created_at TIMESTAMP         -- When event was received
```

#### orders

```sql
id UUID PRIMARY KEY
payment_provider VARCHAR(50)     -- 'stripe' or 'square'
payment_id VARCHAR(255)          -- Provider's payment ID
payment_intent_id VARCHAR(255)   -- Stripe PaymentIntent ID
customer_email VARCHAR(255)      -- Customer email
customer_name VARCHAR(255)       -- Customer name
total_amount BIGINT              -- Amount in cents
currency VARCHAR(10)             -- e.g. 'USD'
status VARCHAR(50)               -- 'pending', 'completed', 'failed', 'refunded'
webhook_event_id UUID            -- Reference to webhook_events
created_at TIMESTAMP
updated_at TIMESTAMP
```

#### order_items

```sql
id UUID PRIMARY KEY
order_id UUID                    -- Reference to orders
product_id INTEGER               -- Reference to products
product_name VARCHAR(255)        -- Product name at time of purchase
product_description TEXT         -- Product description
quantity INTEGER                 -- Quantity ordered
unit_price BIGINT               -- Price per unit in cents
total_price BIGINT              -- quantity * unit_price
created_at TIMESTAMP
```

## Support

For issues or questions:
- Check backend logs: `docker compose logs -f backend`
- Check database: `docker compose exec db psql -U postgres -d ecommerce`
- Stripe documentation: https://stripe.com/docs/webhooks
- Square documentation: https://developer.squareup.com/docs/webhooks/overview
