import React from 'react';
import type { Product } from './ProductList';

export interface CartItem extends Product {
  quantity: number;
}

interface CartProps {
  items: CartItem[];
  onUpdateQuantity: (id: number, quantity: number) => void;
  onRemove: (id: number) => void;
}

export function Cart({ items, onUpdateQuantity, onRemove }: CartProps) {
  const subtotal = items.reduce((sum, item) => sum + item.price * item.quantity, 0);
  return (
    <div>
      <h2>Shopping Cart</h2>
      {items.length === 0 ? (
        <p>Your cart is empty.</p>
      ) : (
        <ul>
          {items.map(item => (
            <li key={item.id}>
              {item.name} x
              <input
                type="number"
                min={1}
                value={item.quantity}
                onChange={e => onUpdateQuantity(item.id, Number(e.target.value))}
                style={{ width: 40, margin: '0 8px' }}
              />
              - ${item.price * item.quantity}
              <button style={{ marginLeft: 8 }} onClick={() => onRemove(item.id)}>
                Remove
              </button>
            </li>
          ))}
        </ul>
      )}
      <p><strong>Subtotal:</strong> ${subtotal}</p>
    </div>
  );
} 