import React, { useEffect, useState } from 'react';

interface Product {
  id: number;
  name: string;
  description?: string;
  price: number;
  inventory: number;
  created_at: string;
}

interface ProductInput {
  name: string;
  description?: string;
  price: number;
  inventory: number;
}

export function AdminProducts({ token }: { token: string }) {
  const [products, setProducts] = useState<Product[]>([]);
  const [form, setForm] = useState<ProductInput>({ name: '', price: 0, inventory: 0 });
  const [editing, setEditing] = useState<Product | null>(null);
  const [error, setError] = useState<string | null>(null);

  const fetchProducts = async () => {
    const res = await fetch('/api/admin/products', {
      headers: { Authorization: `Bearer ${token}` },
    });
    setProducts(await res.json());
  };

  useEffect(() => { fetchProducts(); }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    const method = editing ? 'PUT' : 'POST';
    const url = editing ? `/api/admin/products/${editing.id}` : '/api/admin/products';
    const res = await fetch(url, {
      method,
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${token}`,
      },
      body: JSON.stringify(form),
    });
    if (!res.ok) {
      setError('Failed to save product');
      return;
    }
    setForm({ name: '', price: 0, inventory: 0 });
    setEditing(null);
    fetchProducts();
  };

  const handleEdit = (p: Product) => {
    setEditing(p);
    setForm({ name: p.name, description: p.description, price: p.price, inventory: p.inventory });
  };

  const handleDelete = async (id: number) => {
    if (!window.confirm('Delete this product?')) return;
    await fetch(`/api/admin/products/${id}`, {
      method: 'DELETE',
      headers: { Authorization: `Bearer ${token}` },
    });
    fetchProducts();
  };

  return (
    <div>
      <h2>Products</h2>
      <form onSubmit={handleSubmit} style={{ marginBottom: 16 }}>
        <input placeholder="Name" value={form.name} onChange={e => setForm(f => ({ ...f, name: e.target.value }))} required />
        <input placeholder="Description" value={form.description || ''} onChange={e => setForm(f => ({ ...f, description: e.target.value }))} />
        <input type="number" placeholder="Price" value={form.price} onChange={e => setForm(f => ({ ...f, price: Number(e.target.value) }))} required />
        <input type="number" placeholder="Inventory" value={form.inventory} onChange={e => setForm(f => ({ ...f, inventory: Number(e.target.value) }))} required />
        <button type="submit">{editing ? 'Update' : 'Add'} Product</button>
        {editing && <button type="button" onClick={() => { setEditing(null); setForm({ name: '', price: 0, inventory: 0 }); }}>Cancel</button>}
      </form>
      {error && <div style={{ color: 'red' }}>{error}</div>}
      <ul>
        {products.map(p => (
          <li key={p.id}>
            <strong>{p.name}</strong> (${p.price}) - {p.inventory} in stock
            <button onClick={() => handleEdit(p)} style={{ marginLeft: 8 }}>Edit</button>
            <button onClick={() => handleDelete(p.id)} style={{ marginLeft: 8 }}>Delete</button>
          </li>
        ))}
      </ul>
    </div>
  );
} 