// Product card component for displaying products in a grid

use leptos::*;
use leptos_router::*;
use crate::types::Product;

#[component]
pub fn ProductCard(product: Product) -> impl IntoView {
    let product_clone = product.clone();

    view! {
        <div class="product-card card">
            <A href=format!("/product/{}", product.id) class="product-link">
                // Product image
                <div class="product-image">
                    <img
                        src={product.image_url()}
                        alt={product.name.clone()}
                        loading="lazy"
                    />
                </div>

                // Product details
                <div class="product-details">
                    <h3 class="product-name">{product.name.clone()}</h3>

                    <Show
                        when=move || product_clone.description.is_some()
                        fallback=|| view! { <span></span> }
                    >
                        <p class="product-description">
                            {crate::utils::truncate(
                                product_clone.description.as_ref().unwrap(),
                                80
                            )}
                        </p>
                    </Show>

                    // Price and stock
                    <div class="product-footer">
                        <span class="price">{product.formatted_price()}</span>
                        <span class={format!("badge {}", product.stock_status_class())}>
                            {product.stock_status()}
                        </span>
                    </div>
                </div>
            </A>

            <style>
                {r#"
                .product-card {
                    transition: all var(--transition-base);
                    height: 100%;
                    display: flex;
                    flex-direction: column;
                }

                .product-link {
                    text-decoration: none;
                    color: inherit;
                    display: flex;
                    flex-direction: column;
                    height: 100%;
                }

                .product-image {
                    width: 100%;
                    aspect-ratio: 4 / 3;
                    overflow: hidden;
                    border-radius: var(--radius-md);
                    margin-bottom: var(--spacing-md);
                }

                .product-image img {
                    width: 100%;
                    height: 100%;
                    object-fit: cover;
                    transition: transform var(--transition-base);
                }

                .product-card:hover .product-image img {
                    transform: scale(1.05);
                }

                .product-details {
                    flex: 1;
                    display: flex;
                    flex-direction: column;
                }

                .product-name {
                    font-size: 1.125rem;
                    margin-bottom: var(--spacing-sm);
                    color: var(--color-gray-900);
                }

                .product-description {
                    font-size: 0.875rem;
                    color: var(--color-gray-600);
                    margin-bottom: var(--spacing-md);
                    flex: 1;
                }

                .product-footer {
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    margin-top: auto;
                }

                .badge-warning {
                    background: var(--color-warning);
                    color: white;
                }
                "#}
            </style>
        </div>
    }
}
