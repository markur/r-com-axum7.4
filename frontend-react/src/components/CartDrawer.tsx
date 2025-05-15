import { Box, Typography, IconButton, Button, List, ListItem, ListItemText, TextField } from '@mui/material';
import DeleteIcon from '@mui/icons-material/Delete';
import type { Product } from './ProductGrid';

export interface CartItem extends Product {
  quantity: number;
}

interface CartDrawerProps {
  cart: CartItem[];
  onUpdateQuantity: (id: number, quantity: number) => void;
  onRemove: (id: number) => void;
  onCheckout: () => void;
}

export default function CartDrawer({ cart, onUpdateQuantity, onRemove, onCheckout }: CartDrawerProps) {
  const subtotal = cart.reduce((sum, item) => sum + item.price * item.quantity, 0);
  return (
    <Box sx={{ width: 350, p: 2 }}>
      <Typography variant="h6">Shopping Cart</Typography>
      <List>
        {cart.map(item => (
          <ListItem key={item.id} secondaryAction={
            <IconButton edge="end" onClick={() => onRemove(item.id)}>
              <DeleteIcon />
            </IconButton>
          }>
            <ListItemText
              primary={
                <>
                  {item.name} x
                  <TextField
                    type="number"
                    size="small"
                    value={item.quantity}
                    onChange={e => onUpdateQuantity(item.id, Number(e.target.value))}
                    inputProps={{ min: 1, style: { width: 40, marginLeft: 8 } }}
                    sx={{ width: 60, ml: 1 }}
                  />
                </>
              }
              secondary={`$${item.price * item.quantity}`}
            />
          </ListItem>
        ))}
      </List>
      <Typography sx={{ mt: 2 }}><strong>Subtotal:</strong> ${subtotal}</Typography>
      <Button variant="contained" fullWidth sx={{ mt: 2 }} onClick={onCheckout} disabled={cart.length === 0}>
        Checkout
      </Button>
    </Box>
  );
} 