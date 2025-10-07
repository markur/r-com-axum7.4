#!/bin/bash

# Test Square Integration Script
# This script tests the Square payment integration with your actual credentials

echo "🔧 Testing Square Payment Integration..."
echo "========================================"

# Test 1: Health check
echo "1. Testing health check endpoint..."
curl -s http://localhost:3000/ && echo " ✅ Health check passed" || echo " ❌ Health check failed"

echo ""

# Test 2: Square payment with test card nonce
echo "2. Testing Square payment with test card nonce..."
RESPONSE=$(curl -s -X POST http://localhost:3000/api/square/create-payment \
  -H "Content-Type: application/json" \
  -d '{
    "amount_money": {
      "amount": 100,
      "currency": "USD"
    },
    "source_id": "cnon:card-nonce-ok",
    "idempotency_key": "test-'$(date +%s)'"
  }')

echo "Response: $RESPONSE"

# Check if payment was successful
if echo "$RESPONSE" | grep -q "payment_id"; then
    echo " ✅ Square payment test passed"
    PAYMENT_ID=$(echo "$RESPONSE" | grep -o '"payment_id":"[^"]*"' | cut -d'"' -f4)
    echo "   Payment ID: $PAYMENT_ID"
else
    echo " ❌ Square payment test failed"
    echo "   Error: $RESPONSE"
fi

echo ""

# Test 3: Square payment with declined card nonce
echo "3. Testing Square payment with declined card..."
DECLINE_RESPONSE=$(curl -s -X POST http://localhost:3000/api/square/create-payment \
  -H "Content-Type: application/json" \
  -d '{
    "amount_money": {
      "amount": 100,
      "currency": "USD"
    },
    "source_id": "cnon:card-nonce-declined",
    "idempotency_key": "test-decline-'$(date +%s)'"
  }')

echo "Decline Response: $DECLINE_RESPONSE"

if echo "$DECLINE_RESPONSE" | grep -q "DECLINED\|error"; then
    echo " ✅ Square decline test passed (correctly declined)"
else
    echo " ❌ Square decline test unexpected result"
fi

echo ""

# Test 4: Test with custom location ID
echo "4. Testing with explicit location ID..."
LOCATION_RESPONSE=$(curl -s -X POST http://localhost:3000/api/square/create-payment \
  -H "Content-Type: application/json" \
  -d '{
    "amount_money": {
      "amount": 250,
      "currency": "USD"
    },
    "source_id": "cnon:card-nonce-ok",
    "location_id": "LP7V5561FPK0B",
    "idempotency_key": "test-location-'$(date +%s)'"
  }')

echo "Location Response: $LOCATION_RESPONSE"

if echo "$LOCATION_RESPONSE" | grep -q "payment_id"; then
    echo " ✅ Square payment with location ID passed"
else
    echo " ❌ Square payment with location ID failed"
fi

echo ""
echo "🎯 Square Integration Test Complete!"
echo "========================================"

# Test 5: Show available Square test card nonces
echo ""
echo "📋 Available Square Test Card Nonces:"
echo "   cnon:card-nonce-ok                    - Successful payment"
echo "   cnon:card-nonce-declined              - Declined payment"
echo "   cnon:card-nonce-insufficient-funds    - Insufficient funds"
echo "   cnon:card-nonce-cvv-failure           - CVV failure"
echo "   cnon:card-nonce-avs-failure           - AVS failure"
echo "   cnon:card-nonce-rejected              - Rejected payment"
echo ""
echo "💡 Use these nonces to test different payment scenarios!"