-- Create webhook_events table for logging all incoming webhook events
CREATE TABLE IF NOT EXISTS webhook_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider VARCHAR(50) NOT NULL, -- 'stripe' or 'square'
    event_type VARCHAR(100) NOT NULL, -- e.g. 'payment_intent.succeeded', 'payment.updated'
    event_id VARCHAR(255) NOT NULL UNIQUE, -- Unique event ID from provider (for idempotency)
    payload JSONB NOT NULL, -- Full webhook payload for debugging/auditing
    processed BOOLEAN NOT NULL DEFAULT FALSE, -- Whether we've processed this event
    processed_at TIMESTAMP WITH TIME ZONE,
    error_message TEXT, -- Store any processing errors
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create orders table for tracking completed purchases
CREATE TABLE IF NOT EXISTS orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    payment_provider VARCHAR(50) NOT NULL, -- 'stripe' or 'square'
    payment_id VARCHAR(255) NOT NULL, -- Payment ID from provider
    payment_intent_id VARCHAR(255), -- Stripe PaymentIntent ID (NULL for Square)
    customer_email VARCHAR(255), -- Customer email for order confirmation
    customer_name VARCHAR(255), -- Customer name
    total_amount BIGINT NOT NULL, -- Total amount in cents
    currency VARCHAR(10) NOT NULL DEFAULT 'USD',
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- 'pending', 'completed', 'failed', 'refunded'
    webhook_event_id UUID REFERENCES webhook_events(id), -- Link to triggering webhook event
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create order_items table for tracking individual products in each order
CREATE TABLE IF NOT EXISTS order_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    product_id INTEGER REFERENCES products(id), -- NULL if product was deleted
    product_name VARCHAR(255) NOT NULL, -- Store name to preserve order history
    product_description TEXT,
    quantity INTEGER NOT NULL DEFAULT 1,
    unit_price BIGINT NOT NULL, -- Price per unit in cents at time of purchase
    total_price BIGINT NOT NULL, -- quantity * unit_price
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for common queries
CREATE INDEX IF NOT EXISTS idx_webhook_events_provider_event_type ON webhook_events(provider, event_type);
CREATE INDEX IF NOT EXISTS idx_webhook_events_processed ON webhook_events(processed);
CREATE INDEX IF NOT EXISTS idx_webhook_events_created_at ON webhook_events(created_at);
CREATE INDEX IF NOT EXISTS idx_orders_payment_provider_payment_id ON orders(payment_provider, payment_id);
CREATE INDEX IF NOT EXISTS idx_orders_customer_email ON orders(customer_email);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);
CREATE INDEX IF NOT EXISTS idx_orders_created_at ON orders(created_at);
CREATE INDEX IF NOT EXISTS idx_order_items_order_id ON order_items(order_id);
CREATE INDEX IF NOT EXISTS idx_order_items_product_id ON order_items(product_id);
