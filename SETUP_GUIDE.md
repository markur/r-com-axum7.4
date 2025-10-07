# Setup Guide: Square Payments & Letre Email Marketing

## Square Payments Setup

### 1. Create Square Developer Account
1. Go to [Square Developer Dashboard](https://developer.squareup.com/)
2. Sign up or log in with your Square account
3. Create a new application

### 2. Get API Credentials
1. In your Square app dashboard, go to **Credentials**
2. Copy the following values:
   - **Application ID** (starts with `sq0idp-`)
   - **Access Token** for Sandbox (starts with `EAAAl...`)
   - **Location ID** (from Locations tab)

### 3. Configure Environment Variables
Add to your `.env` file:
```env
SQUARE_ACCESS_TOKEN=EAAAl_your_sandbox_access_token_here
SQUARE_APPLICATION_ID=sq0idp-your_application_id_here
SQUARE_ENVIRONMENT=sandbox
```

### 4. Frontend Integration
Install Square Web Payments SDK in your frontend:
```bash
npm install @square/web-payments-sdk-js
```

Example frontend code:
```javascript
import { payments } from '@square/web-payments-sdk-js';

async function initializeSquare() {
  const paymentsInstance = payments('YOUR_APPLICATION_ID', 'YOUR_LOCATION_ID');
  
  const card = await paymentsInstance.card();
  await card.attach('#card-container');
  
  return { paymentsInstance, card };
}

async function processPayment(card, amount) {
  const tokenResult = await card.tokenize();
  
  if (tokenResult.status === 'OK') {
    const response = await fetch('/api/square/create-payment', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        amount_money: { amount: amount * 100, currency: 'USD' },
        source_id: tokenResult.token,
        location_id: 'YOUR_LOCATION_ID'
      })
    });
    
    return response.json();
  }
}
```

---

## Letre Email Marketing Setup

### 1. Create Letre Account
1. Go to [Letre.io](https://letre.io) (or your Letre provider)
2. Sign up for an account
3. Complete email verification

### 2. Get API Credentials
1. In your Letre dashboard, go to **Settings** → **API Keys**
2. Generate a new API key
3. Copy the API key (starts with `letre_`)

### 3. Configure Environment Variables
Add to your `.env` file:
```env
LETRE_API_KEY=letre_your_api_key_here
LETRE_API_URL=https://api.letre.io
```

### 4. Create Email Templates
In your Letre dashboard, create these templates:

#### Order Confirmation Template
- **Template ID**: `order_confirmation`
- **Subject**: `Order Confirmation - {{order_number}}`
- **Content**:
```html
<h1>Thank you for your order!</h1>
<p>Hi {{first_name}},</p>
<p>Your order {{order_number}} has been confirmed.</p>
<p><strong>Total: {{total_amount}}</strong></p>
<h3>Items:</h3>
<ul>
{{#each items}}
  <li>{{name}} - Qty: {{quantity}} - {{price}}</li>
{{/each}}
</ul>
<p>We'll send you tracking information once your order ships.</p>
```

#### Welcome Email Template
- **Template ID**: `welcome`
- **Subject**: `Welcome to our store!`
- **Content**:
```html
<h1>Welcome {{first_name}}!</h1>
<p>Thanks for subscribing to our newsletter.</p>
<p>You'll be the first to know about new products and special offers.</p>
```

#### Abandoned Cart Template
- **Template ID**: `abandoned_cart`
- **Subject**: `Don't forget your items!`
- **Content**:
```html
<h1>You left something behind...</h1>
<p>Hi {{first_name}},</p>
<p>You have {{item_count}} items waiting in your cart.</p>
<p>Complete your purchase now and get free shipping!</p>
<a href="{{cart_url}}">Complete Purchase</a>
```

---

## Testing Your Setup

### 1. Test Square Payments
```bash
curl -X POST http://localhost:3000/api/square/create-payment \
  -H "Content-Type: application/json" \
  -d '{
    "amount_money": {"amount": 100, "currency": "USD"},
    "source_id": "cnon:card-nonce-ok",
    "location_id": "YOUR_LOCATION_ID"
  }'
```

### 2. Test Email Subscription
```bash
curl -X POST http://localhost:3000/api/email/subscribe \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "first_name": "Test",
    "source": "api_test"
  }'
```

### 3. Test Triggered Email
```bash
curl -X POST http://localhost:3000/api/email/trigger \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "template_id": "order_confirmation",
    "variables": {
      "order_number": "TEST-001",
      "total_amount": "$10.00",
      "first_name": "Test",
      "items": [{"name": "Test Product", "quantity": 1, "price": "$10.00"}]
    }
  }'
```

---

## Production Deployment

### Square Production Setup
1. In Square Dashboard, switch to **Production** tab
2. Get production credentials:
   - Production Access Token
   - Production Application ID
3. Update environment variables:
```env
SQUARE_ENVIRONMENT=production
SQUARE_ACCESS_TOKEN=your_production_token
```

### Letre Production Setup
1. Verify your domain in Letre dashboard
2. Set up SPF/DKIM records for better deliverability
3. Configure webhook endpoints for email events

### Security Considerations
- Use environment variables for all API keys
- Enable HTTPS in production
- Implement rate limiting for API endpoints
- Set up monitoring and logging
- Use strong JWT secrets

---

## Common Issues & Solutions

### Square Issues
**Error: "Invalid location_id"**
- Solution: Get location ID from Square Dashboard → Locations

**Error: "Invalid source_id"**
- Solution: Ensure you're using a valid card nonce from Square SDK

### Letre Issues
**Error: "Unauthorized"**
- Solution: Check API key is correct and has proper permissions

**Error: "Template not found"**
- Solution: Verify template ID exists in Letre dashboard

### General Issues
**Error: "Client not configured"**
- Solution: Check environment variables are set correctly

**Database connection errors**
- Solution: Ensure PostgreSQL is running and DATABASE_URL is correct

---

## Next Steps

1. **Set up webhooks** for payment confirmations
2. **Implement cart abandonment** email automation
3. **Add customer segmentation** for targeted campaigns
4. **Set up analytics** tracking for email performance
5. **Implement A/B testing** for email templates