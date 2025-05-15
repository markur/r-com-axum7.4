/**
 * CheckoutDialog
 *
 * A Material-UI Dialog component that handles Stripe payment for the cart subtotal.
 * Uses Stripe Elements for secure card entry and communicates with the backend
 * to create a payment intent and confirm the payment.
 *
 * Props:
 *   - open: boolean, whether the dialog is open
 *   - onClose: function, called to close the dialog
 *   - amount: number, the total amount (in cents) to charge
 */
import React, { useState } from 'react';
import { Dialog, DialogTitle, DialogContent, DialogActions, Button, Typography, Box } from '@mui/material';
import { loadStripe } from '@stripe/stripe-js';
import { Elements, CardElement, useStripe, useElements } from '@stripe/react-stripe-js';

// Load Stripe public key from environment variable
const stripePromise = loadStripe(import.meta.env.VITE_STRIPE_PUBLIC_KEY as string);

interface CheckoutDialogProps {
  open: boolean;
  onClose: () => void;
  amount: number;
}

// Inner form component for handling payment logic
function CheckoutForm({ amount, onClose }: { amount: number; onClose: () => void }) {
  const stripe = useStripe();
  const elements = useElements();
  const [status, setStatus] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  // Handle form submission and payment
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setStatus(null);
    // 1. Create payment intent on the backend
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
    // 2. Confirm card payment with Stripe
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
      setTimeout(onClose, 2000); // Close dialog after success
    } else {
      setStatus('Payment failed');
    }
    setLoading(false);
  };

  return (
    <form onSubmit={handleSubmit}>
      <Typography gutterBottom>Amount: <strong>${(amount / 100).toFixed(2)}</strong></Typography>
      <Box sx={{ my: 2 }}>
        <CardElement options={{ style: { base: { fontSize: '18px' } } }} />
      </Box>
      <DialogActions>
        <Button onClick={onClose} disabled={loading}>Cancel</Button>
        <Button type="submit" variant="contained" disabled={!stripe || loading}>
          {loading ? 'Processing...' : 'Pay'}
        </Button>
      </DialogActions>
      {status && <Typography color={status.includes('success') ? 'green' : 'red'} sx={{ mt: 2 }}>{status}</Typography>}
    </form>
  );
}

// Main dialog component
export default function CheckoutDialog({ open, onClose, amount }: CheckoutDialogProps) {
  return (
    <Dialog open={open} onClose={onClose} maxWidth="xs" fullWidth>
      <DialogTitle>Checkout</DialogTitle>
      <DialogContent>
        <Elements stripe={stripePromise}>
          <CheckoutForm amount={amount} onClose={onClose} />
        </Elements>
      </DialogContent>
    </Dialog>
  );
} 