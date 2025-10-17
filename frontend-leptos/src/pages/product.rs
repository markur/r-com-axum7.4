// Individual product detail page

use leptos::*;
use leptos_router::*;
use crate::{
    api::{
        products::fetch_products,
        cart::{load_cart, add_to_cart},
    },
    types::Product,
};

#[component]
pub fn ProductPage() -> impl IntoView {
    let params = use_params_map();
    let navigate = use_navigate();

    // Cart state
    let cart = create_rw_signal(load_cart());

    // Get product ID from URL
    let product_id = move || {
        params.with(|p| {
            p.get("id")
                .and_then(|id| id.parse::<i32>().ok())
        })
    };

    // Fetch all products and filter for the one we want
    // (In production, you'd have a fetch_product_by_id endpoint)
    let product = create_resource(
        product_id,
        |id| async move {
            id.and_then(|product_id| {
                fetch_products().await.ok()
                    .and_then(|products| {
                        products.into_iter()
                            .find(|p| p.id == product_id)
                    })
            })
        },
    );

    // Quantity selector
    let (quantity, set_quantity) = create_signal(1u32);

    // Add to cart handler
    let handle_add_to_cart = move |product: Product| {
        let mut current_cart = cart.get();
        add_to_cart(&mut current_cart, product.clone(), quantity.get());
        cart.set(current_cart);

        // Show success feedback (could use a toast notification)
        log::info!("Added {} x {} to cart", quantity.get(), product.name);

        // Navigate to cart
        navigate("/cart", Default::default());
    };

    view! {
        <div class="product-page container">
            <Suspense fallback=move || view! {
                <div class="loading">
                    <div class="spinner"></div>
                    <p>"Loading product..."</p>
                </div>
            }>
                {move || {
                    product.get().map(|opt_product| {
                        match opt_product {
                            Some(product) => {
                                let product_clone = product.clone();
                                view! {
                                    <div class="product-detail">
                                        // Breadcrumb
                                        <nav class="breadcrumb">
                                            <A href="/">"Home"</A>
                                            <span>" / "</span>
                                            <A href="/catalog">"Shop"</A>
                                            <span>" / "</span>
                                            <span>{product.name.clone()}</span>
                                        </nav>

                                        <div class="product-content">
                                            // Product image
                                            <div class="product-image-large">
                                                <img
                                                    src={product.image_url()}
                                                    alt={product.name.clone()}
                                                />
                                            </div>

                                            // Product info
                                            <div class="product-info">
                                                <h1>{product.name.clone()}</h1>

                                                <div class="product-meta">
                                                    <span class="price-large">{product.formatted_price()}</span>
                                                    <span class={format!("badge {}", product.stock_status_class())}>
                                                        {product.stock_status()}
                                                    </span>
                                                </div>

                                                <Show
                                                    when=move || product_clone.description.is_some()
                                                    fallback=|| view! { <span></span> }
                                                >
                                                    <div class="product-description">
                                                        <h3>"Description"</h3>
                                                        <p>{product_clone.description.clone()}</p>
                                                    </div>
                                                </Show>

                                                // Add to cart section
                                                <Show
                                                    when=move || product.is_in_stock()
                                                    fallback=|| view! {
                                                        <div class="out-of-stock">
                                                            <p>"This product is currently out of stock."</p>
                                                        </div>
                                                    }
                                                >
                                                    <div class="add-to-cart-section">
                                                        // Quantity selector
                                                        <div class="quantity-selector">
                                                            <label>"Quantity:"</label>
                                                            <div class="quantity-controls">
                                                                <button
                                                                    class="btn btn-sm"
                                                                    on:click=move |_| set_quantity.update(|q| *q = (*q).saturating_sub(1).max(1))
                                                                    disabled=move || quantity.get() <= 1
                                                                >
                                                                    "-"
                                                                </button>
                                                                <span class="quantity-value">{quantity}</span>
                                                                <button
                                                                    class="btn btn-sm"
                                                                    on:click=move |_| set_quantity.update(|q| *q = (*q + 1).min(product.inventory as u32))
                                                                    disabled=move || quantity.get() >= product.inventory as u32
                                                                >
                                                                    "+"
                                                                </button>
                                                            </div>
                                                        </div>

                                                        // Add to cart button
                                                        <button
                                                            class="btn btn-primary btn-lg add-to-cart-btn"
                                                            on:click=move |_| handle_add_to_cart(product.clone())
                                                        >
                                                            "Add to Cart"
                                                        </button>
                                                    </div>
                                                </Show>
                                            </div>
                                        </div>
                                    </div>
                                }.into_view()
                            }
                            None => {
                                view! {
                                    <div class="error-state">
                                        <h2>"Product Not Found"</h2>
                                        <p>"Sorry, we couldn't find that product."</p>
                                        <A href="/catalog" class="btn btn-primary">"Back to Shop"</A>
                                    </div>
                                }.into_view()
                            }
                        }
                    })
                }}
            </Suspense>

            <style>
                {r#"
                .product-page {
                    padding: var(--spacing-xl) 0;
                }

                .breadcrumb {
                    margin-bottom: var(--spacing-lg);
                    font-size: 0.875rem;
                    color: var(--color-gray-600);
                }

                .breadcrumb a {
                    color: var(--color-primary);
                }

                .product-content {
                    display: grid;
                    grid-template-columns: 1fr 1fr;
                    gap: var(--spacing-2xl);
                    margin-top: var(--spacing-xl);
                }

                .product-image-large {
                    width: 100%;
                    aspect-ratio: 1;
                    overflow: hidden;
                    border-radius: var(--radius-lg);
                    background: var(--color-gray-100);
                }

                .product-image-large img {
                    width: 100%;
                    height: 100%;
                    object-fit: cover;
                }

                .product-info h1 {
                    font-size: 2.5rem;
                    margin-bottom: var(--spacing-md);
                }

                .product-meta {
                    display: flex;
                    align-items: center;
                    gap: var(--spacing-md);
                    margin-bottom: var(--spacing-xl);
                }

                .price-large {
                    font-size: 2rem;
                    font-weight: 700;
                    color: var(--color-primary);
                }

                .product-description {
                    margin-bottom: var(--spacing-xl);
                    padding-bottom: var(--spacing-xl);
                    border-bottom: 1px solid var(--color-gray-200);
                }

                .product-description h3 {
                    font-size: 1.25rem;
                    margin-bottom: var(--spacing-sm);
                }

                .product-description p {
                    color: var(--color-gray-700);
                    line-height: 1.8;
                }

                .add-to-cart-section {
                    display: flex;
                    flex-direction: column;
                    gap: var(--spacing-md);
                }

                .quantity-selector {
                    display: flex;
                    flex-direction: column;
                    gap: var(--spacing-sm);
                }

                .quantity-controls {
                    display: flex;
                    align-items: center;
                    gap: var(--spacing-md);
                }

                .quantity-value {
                    font-size: 1.25rem;
                    font-weight: 600;
                    min-width: 50px;
                    text-align: center;
                }

                .add-to-cart-btn {
                    width: 100%;
                }

                .out-of-stock {
                    background: var(--color-gray-100);
                    padding: var(--spacing-lg);
                    border-radius: var(--radius-md);
                    text-align: center;
                }

                .loading,
                .error-state {
                    text-align: center;
                    padding: var(--spacing-2xl);
                }

                .badge-warning {
                    background: var(--color-warning);
                    color: white;
                }

                @media (max-width: 768px) {
                    .product-content {
                        grid-template-columns: 1fr;
                    }

                    .product-info h1 {
                        font-size: 2rem;
                    }

                    .price-large {
                        font-size: 1.5rem;
                    }
                }
                "#}
            </style>
        </div>
    }
}
