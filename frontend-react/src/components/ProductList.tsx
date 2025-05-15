import React, { useEffect, useState } from 'react';

export interface Product {
  id: number;
  name: string;
  description?: string;
  price: number;
  inventory: number;
}

interface ProductListProps {
  onAddToCart: (product: Product) => void;
}

export function ProductList({ onAddToCart }: ProductListProps) {
  const [products, setProducts] = useState<Product[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetch('/api/products')
      .then(res => res.json())
      .then(data => {
        setProducts(data);
        setLoading(false);
      });
  }, []);

  if (loading) return <div>Loading products...</div>;

  return (
    <div>
      <h2>Products</h2>
      <ul>
        {products.map(product => (
          <li key={product.id}>
            <strong>{product.name}</strong> - ${product.price}
            <button style={{ marginLeft: 8 }} onClick={() => onAddToCart(product)}>
              Add to Cart
            </button>
            <div style={{ fontSize: '0.9em', color: '#555' }}>{product.description}</div>
          </li>
        ))}
      </ul>
    </div>
  );
} 