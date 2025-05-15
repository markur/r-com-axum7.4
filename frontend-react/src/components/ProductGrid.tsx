import { Grid, Card, CardContent, CardActions, Button, Typography } from '@mui/material';

export interface Product {
  id: number;
  name: string;
  description?: string;
  price: number;
  inventory: number;
  created_at?: string;
}

interface ProductGridProps {
  products: Product[];
  onAddToCart: (product: Product) => void;
}

export default function ProductGrid({ products, onAddToCart }: ProductGridProps) {
  return (
    <Grid container spacing={3}>
      {products.map(product => (
        <Grid item xs={12} sm={6} md={4} key={product.id}>
          <Card>
            {/* Optionally add product image here */}
            <CardContent>
              <Typography variant="h6">{product.name}</Typography>
              <Typography color="text.secondary">${product.price}</Typography>
              <Typography variant="body2">{product.description}</Typography>
            </CardContent>
            <CardActions>
              <Button onClick={() => onAddToCart(product)} variant="contained">Add to Cart</Button>
            </CardActions>
          </Card>
        </Grid>
      ))}
    </Grid>
  );
} 