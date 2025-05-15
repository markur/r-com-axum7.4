import { useState, useEffect } from 'react';
import { AppBar, Toolbar, Typography, IconButton, Drawer, Snackbar, Container, CircularProgress, Box } from '@mui/material';
import ShoppingCartIcon from '@mui/icons-material/ShoppingCart';
import ProductGrid from './components/ProductGrid';
import CartDrawer from './components/CartDrawer';
import CheckoutDialog from './components/CheckoutDialog';
import type { Product } from './components/ProductGrid';
import type { CartItem } from './components/CartDrawer';

export default function App() {
  // State for cart drawer
  const [cartOpen, setCartOpen] = useState(false);
  // State for snackbar notifications
  const [snackbar, setSnackbar] = useState({ open: false, message: '' });
  // State for cart items (typed)
  const [cart, setCart] = useState<CartItem[]>([] as CartItem[]);
  // State for products (typed)
  const [products, setProducts] = useState<Product[]>([] as Product[]);
  // State for loading and error
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  // State for checkout dialog
  const [checkoutOpen, setCheckoutOpen] = useState(false);

  // Fetch products from backend API on mount
  useEffect(() => {
    setLoading(true);
    setError(null);
    fetch('/api/products')
      .then(res => {
        if (!res.ok) throw new Error('Failed to fetch products');
        return res.json();
      })
      .then(data => {
        setProducts(data);
        setLoading(false);
      })
      .catch(err => {
        setError(err.message);
        setLoading(false);
      });
  }, []);

  // Calculate cart subtotal in cents
  const subtotal = cart.reduce((sum, item) => sum + item.price * item.quantity, 0) * 100;

  // Handler to add a product to the cart
  const handleAddToCart = (product: Product) => {
    setCart((prev: CartItem[]) => {
      const existing = prev.find(item => item.id === product.id);
      if (existing) {
        return prev.map(item =>
          item.id === product.id ? { ...item, quantity: item.quantity + 1 } : item
        );
      } else {
        return [...prev, { ...product, quantity: 1 }];
      }
    });
    setSnackbar({ open: true, message: `${product.name} added to cart!` });
  };

  // Handler to update quantity of a cart item
  const handleUpdateQuantity = (id: number, quantity: number) => {
    setCart((prev: CartItem[]) => prev.map(item => item.id === id ? { ...item, quantity } : item));
  };

  // Handler to remove an item from the cart
  const handleRemove = (id: number) => {
    setCart((prev: CartItem[]) => prev.filter(item => item.id !== id));
  };

  // Handler to open the checkout dialog
  const handleCheckout = () => {
    setCheckoutOpen(true);
  };

  // Handler to close the checkout dialog
  const handleCloseCheckout = () => {
    setCheckoutOpen(false);
    setCart([]); // Optionally clear cart on successful payment
  };

  return (
    <>
      <AppBar position="static">
        <Toolbar>
          <Typography variant="h6" sx={{ flexGrow: 1 }}>Rust E-commerce Platform</Typography>
          <IconButton color="inherit" onClick={() => setCartOpen(true)}>
            <ShoppingCartIcon />
          </IconButton>
        </Toolbar>
      </AppBar>
      <Container sx={{ mt: 4 }}>
        {/* Show loading spinner or error message if needed */}
        {loading ? (
          <Box sx={{ display: 'flex', justifyContent: 'center', mt: 8 }}>
            <CircularProgress />
          </Box>
        ) : error ? (
          <Typography color="error" align="center">{error}</Typography>
        ) : (
          <ProductGrid products={products} onAddToCart={handleAddToCart} />
        )}
      </Container>
      <Drawer anchor="right" open={cartOpen} onClose={() => setCartOpen(false)}>
        <CartDrawer cart={cart} onUpdateQuantity={handleUpdateQuantity} onRemove={handleRemove} onCheckout={handleCheckout} />
      </Drawer>
      {/* Checkout dialog for Stripe payment */}
      <CheckoutDialog open={checkoutOpen} onClose={handleCloseCheckout} amount={subtotal} />
      <Snackbar
        open={snackbar.open}
        autoHideDuration={3000}
        onClose={() => setSnackbar({ ...snackbar, open: false })}
        message={snackbar.message}
      />
    </>
  );
}
