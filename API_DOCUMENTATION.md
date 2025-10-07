# E-commerce Platform API Documentation

## Overview
This API provides endpoints for product management, payment processing (Stripe & Square), and email marketing (Letre).

## Base URL
- Development: `http://localhost:3000`
- Production: `https://your-domain.com`

---

## Authentication
Admin endpoints require JWT authentication. Include the token in the Authorization header:
```
Authorization: Bearer <your_jwt_token>
```

---

## Payment Processing

### Stripe Payments

#### Create Payment Intent
```http
POST /api/create-payment-intent
Content-Type: application/json

{
  "amount": 2000,
  "currency": "USD"
}
```

**Response:**
```json
{
  "client_secret": "pi_1234567890_secret_abcdef"
}
```

### Square Payments

#### Create Square Payment
```http
POST /api/square/create-payment
Content-Type: application/json

{
  "amount_money": {
    "amount": 2000,
    "currency": "USD"
  },
  "source_id": "cnon:card-nonce-from-square-sdk",
  "location_id": "your_square_location_id",
  "idempotency_key": "optional-unique-key"
}
```

**Response:**
```json
{
  "payment_id": "payment_123456789",
  "status": "COMPLETED",
  "receipt_url": "https://squareup.com/receipt/preview/payment_123456789"
}
```

---

## Email Marketing (Letre)

### Subscribe Email
```http
POST /api/email/subscribe
Content-Type: application/json

{
  "email": "customer@example.com",
  "first_name": "John",
  "last_name": "Doe",
  "source": "checkout"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Successfully subscribed customer@example.com",
  "id": null
}
```

### Unsubscribe Email
```http
POST /api/email/unsubscribe
Content-Type: application/json

{
  "email": "customer@example.com"
}
```

### Send Email Campaign
```http
POST /api/email/campaign
Content-Type: application/json

{
  "subject": "New Product Launch!",
  "content": "<h1>Check out our new products</h1><p>Amazing deals await!</p>",
  "recipient_tags": ["customer", "newsletter"],
  "send_immediately": true
}
```

### Trigger Automated Email
```http
POST /api/email/trigger
Content-Type: application/json

{
  "email": "customer@example.com",
  "template_id": "order_confirmation",
  "variables": {
    "order_number": "ORD-12345",
    "total_amount": "$29.99",
    "items": [
      {"name": "Product A", "quantity": 2, "price": "$14.99"}
    ]
  }
}
```

### List Subscribers (Admin)
```http
GET /api/email/subscribers
Authorization: Bearer <admin_jwt_token>
```

---

## Product Management

### Get All Products
```http
GET /api/products
```

**Response:**
```json
[
  {
    "id": 1,
    "name": "Sample Product",
    "description": "A great product",
    "price": 29.99,
    "inventory": 100,
    "created_at": "2023-05-15T10:30:00Z"
  }
]
```

### Admin Product Management

#### List Products (Admin)
```http
GET /api/admin/products
Authorization: Bearer <admin_jwt_token>
```

#### Create Product (Admin)
```http
POST /api/admin/products
Authorization: Bearer <admin_jwt_token>
Content-Type: application/json

{
  "name": "New Product",
  "description": "Product description",
  "price": 49.99,
  "inventory": 50
}
```

#### Update Product (Admin)
```http
PUT /api/admin/products/1
Authorization: Bearer <admin_jwt_token>
Content-Type: application/json

{
  "name": "Updated Product",
  "description": "Updated description",
  "price": 59.99,
  "inventory": 75
}
```

#### Delete Product (Admin)
```http
DELETE /api/admin/products/1
Authorization: Bearer <admin_jwt_token>
```

---

## Admin Authentication

### Register Admin
```http
POST /api/admin/register
Content-Type: application/json

{
  "username": "admin",
  "password": "strongpassword123"
}
```

### Login Admin
```http
POST /api/admin/login
Content-Type: application/json

{
  "username": "admin",
  "password": "strongpassword123"
}
```

**Response (if TOTP not set up):**
```json
{
  "secret": "JBSWY3DPEHPK3PXP",
  "qr_url": "otpauth://totp/AdminPortal:admin?secret=JBSWY3DPEHPK3PXP&issuer=RustEcomAdmin"
}
```

### Verify TOTP
```http
POST /api/admin/totp/verify
Content-Type: application/json

{
  "username": "admin",
  "code": "123456"
}
```

**Response:**
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
}
```

---

## Environment Variables

### Required
- `DATABASE_URL`: PostgreSQL connection string
- `STRIPE_SECRET_KEY`: Stripe secret key
- `SQUARE_ACCESS_TOKEN`: Square access token
- `SQUARE_APPLICATION_ID`: Square application ID
- `LETRE_API_KEY`: Letre API key

### Optional
- `JWT_SECRET`: JWT signing secret (defaults to "supersecretjwtkey")
- `SQUARE_ENVIRONMENT`: "sandbox" or "production" (defaults to "sandbox")
- `LETRE_API_URL`: Letre API base URL (defaults to "https://api.letre.io")

---

## Error Responses

All endpoints return errors in this format:
```json
{
  "error": "Error message description"
}
```

Common HTTP status codes:
- `200`: Success
- `201`: Created
- `400`: Bad Request
- `401`: Unauthorized
- `404`: Not Found
- `500`: Internal Server Error

---

## Integration Examples

### Complete Checkout Flow
1. **Create payment** (Stripe or Square)
2. **Subscribe customer** to email list
3. **Send order confirmation** email

```javascript
// Frontend example
async function completeCheckout(orderData) {
  // 1. Process payment
  const payment = await fetch('/api/square/create-payment', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      amount_money: { amount: orderData.total * 100, currency: 'USD' },
      source_id: orderData.cardNonce,
      location_id: 'your_location_id'
    })
  });

  if (payment.ok) {
    // 2. Subscribe to email list
    await fetch('/api/email/subscribe', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        email: orderData.email,
        first_name: orderData.firstName,
        source: 'checkout'
      })
    });

    // 3. Send order confirmation
    await fetch('/api/email/trigger', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        email: orderData.email,
        template_id: 'order_confirmation',
        variables: {
          order_number: orderData.orderNumber,
          total_amount: `$${orderData.total}`,
          items: orderData.items
        }
      })
    });
  }
}
```

---

## Setup Instructions

1. **Configure environment variables** in `.env` file
2. **Set up Square sandbox account** and get API credentials
3. **Set up Letre account** and get API key
4. **Create email templates** in Letre dashboard
5. **Run database migrations**: `sqlx migrate run`
6. **Start the server**: `cargo run` or `docker-compose up`