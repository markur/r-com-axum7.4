import React, { useState } from 'react';
import { loadStripe } from '@stripe/stripe-js';
import { Elements, CardElement, useStripe, useElements } from '@stripe/react-stripe-js';

const stripePromise = loadStripe(import.meta.env.VITE_STRIPE_PUBLIC_KEY as string);

interface CheckoutProps {
  amount?: number;
}

function CheckoutForm({ amount: propAmount }: CheckoutProps) {
  const stripe = useStripe();
  const elements = useElements();
  const [amount, setAmount] = useState(propAmount ?? 5000); // $50.00 default
  const [status, setStatus] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setStatus(null);
    // 1. Create payment intent
    const res = await fetch('/api/create-payment-intent', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ amount, currency: 'usd' }),
    });
    const data = await res.json();
    if (!data.client_secret) {
      setStatus('Failed to create payment intent');
      setLoading(false);
      return;
    }
    // 2. Confirm card payment
    if (!stripe || !elements) {
      setStatus('Stripe not loaded');
      setLoading(false);
      return;
    }
    const result = await stripe.confirmCardPayment(data.client_secret, {
      payment_method: {
        card: elements.getElement(CardElement)!,
      },
    });
    if (result.error) {
      setStatus(result.error.message || 'Payment failed');
    } else if (result.paymentIntent?.status === 'succeeded') {
      setStatus('Payment successful!');
    } else {
      setStatus('Payment failed');
    }
    setLoading(false);
  };

  return (
    <form onSubmit={handleSubmit}>
      <h2>Checkout</h2>
      <label>
        Amount (cents):
        <input
          type="number"
          value={amount}
          onChange={e => setAmount(Number(e.target.value))}
          min={100}
        />
      </label>
      <div style={{ margin: '1em 0' }}>
        <CardElement />
      </div>
      <button type="submit" disabled={!stripe || loading}>
        {loading ? 'Processing...' : 'Pay'}
      </button>
      {status && <div style={{ marginTop: '1em' }}>{status}</div>}
    </form>
  );
}

export function Checkout(props: CheckoutProps) {
  return (
    <Elements stripe={stripePromise}>
      <CheckoutForm {...props} />
    </Elements>
  );
} 