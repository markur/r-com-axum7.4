// Shopping cart page

use leptos::*;
use leptos_router::*;
use crate::{
    api::cart::{load_cart, save_cart, update_cart_quantity, remove_from_cart},
    types::Cart,
};

#[component]
pub fn CartPage() -> impl IntoView {
    // Load cart from localStorage
    let cart = create_rw_signal(load_cart());

    // Handle quantity update
    let update_quantity = move |product_id: i32, new_quantity: u32| {
        let mut current_cart = cart.get();
        update_cart_quantity(&mut current_cart, product_id, new_quantity);
        cart.set(current_cart);
    };

    // Handle item removal
    let remove_item = move |product_id: i32| {
        let mut current_cart = cart.get();
        remove_from_cart(&mut current_cart, product_id);
        cart.set(current_cart);
    };

    view! {
        <div class="cart-page container">
            <h1 class="page-title">"Shopping Cart"</h1>

            <Show
                when=move || !cart.get().is_empty()
                fallback=|| view! {
                    <div class="empty-cart">
                        <h2>"Your cart is empty"</h2>
                        <p>"Add some products to get started!"</p>
                        <A href="/catalog" class="btn btn-primary">"Shop Now"</A>
                    </div>
                }
            >
                <div class="cart-content">
                    // Cart items
                    <div class="cart-items">
                        {move || {
                            cart.get().items.into_iter().map(|item| {
                                let product_id = item.product.id;
                                let quantity = item.quantity;

                                view! {
                                    <div class="cart-item card">
                                        // Product image
                                        <div class="item-image">
                                            <img
                                                src={item.product.image_url()}
                                                alt={item.product.name.clone()}
                                            />
                                        </div>

                                        // Product details
                                        <div class="item-details">
                                            <h3>{item.product.name.clone()}</h3>
                                            <p class="item-price">{item.product.formatted_price()}</p>
                                        </div>

                                        // Quantity controls
                                        <div class="item-quantity">
                                            <label>"Qty:"</label>
                                            <div class="quantity-controls">
                                                <button
                                                    class="btn btn-sm"
                                                    on:click=move |_| update_quantity(product_id, quantity.saturating_sub(1))
                                                >
                                                    "-"
                                                </button>
                                                <span class="quantity-value">{quantity}</span>
                                                <button
                                                    class="btn btn-sm"
                                                    on:click=move |_| update_quantity(product_id, quantity + 1)
                                                >
                                                    "+"
                                                </button>
                                            </div>
                                        </div>

                                        // Subtotal
                                        <div class="item-subtotal">
                                            <span class="subtotal-label">"Subtotal:"</span>
                                            <span class="subtotal-value">{item.formatted_subtotal()}</span>
                                        </div>

                                        // Remove button
                                        <button
                                            class="btn-remove"
                                            on:click=move |_| remove_item(product_id)
                                            title="Remove item"
                                        >
                                            "×"
                                        </button>
                                    </div>
                                }
                            }).collect_view()
                        }}
                    </div>

                    // Cart summary
                    <div class="cart-summary card">
                        <h3>"Order Summary"</h3>

                        <div class="summary-row">
                            <span>"Subtotal:"</span>
                            <span>{move || cart.get().formatted_subtotal()}</span>
                        </div>

                        <div class="summary-row">
                            <span>"Tax (8%):"</span>
                            <span>{move || cart.get().formatted_tax()}</span>
                        </div>

                        <div class="summary-row summary-total">
                            <span>"Total:"</span>
                            <span>{move || cart.get().formatted_total()}</span>
                        </div>

                        <A href="/checkout" class="btn btn-primary btn-lg checkout-btn">
                            "Proceed to Checkout"
                        </A>

                        <A href="/catalog" class="btn btn-outline continue-shopping">
                            "Continue Shopping"
                        </A>
                    </div>
                </div>
            </Show>

            <style>
                {r#"
                .cart-page {
                    padding: var(--spacing-2xl) 0;
                }

                .page-title {
                    text-align: center;
                    margin-bottom: var(--spacing-xl);
                }

                .empty-cart {
                    text-align: center;
                    padding: var(--spacing-2xl);
                    background: var(--color-gray-100);
                    border-radius: var(--radius-lg);
                }

                .empty-cart h2 {
                    margin-bottom: var(--spacing-md);
                }

                .empty-cart p {
                    color: var(--color-gray-600);
                    margin-bottom: var(--spacing-lg);
                }

                .cart-content {
                    display: grid;
                    grid-template-columns: 2fr 1fr;
                    gap: var(--spacing-xl);
                }

                .cart-items {
                    display: flex;
                    flex-direction: column;
                    gap: var(--spacing-md);
                }

                .cart-item {
                    display: grid;
                    grid-template-columns: 100px 1fr auto auto auto;
                    gap: var(--spacing-md);
                    align-items: center;
                    position: relative;
                }

                .item-image {
                    width: 100px;
                    height: 100px;
                    border-radius: var(--radius-md);
                    overflow: hidden;
                }

                .item-image img {
                    width: 100%;
                    height: 100%;
                    object-fit: cover;
                }

                .item-details h3 {
                    font-size: 1.125rem;
                    margin-bottom: var(--spacing-xs);
                }

                .item-price {
                    color: var(--color-gray-600);
                    margin: 0;
                }

                .item-quantity {
                    display: flex;
                    flex-direction: column;
                    gap: var(--spacing-xs);
                }

                .quantity-controls {
                    display: flex;
                    align-items: center;
                    gap: var(--spacing-sm);
                }

                .quantity-value {
                    font-weight: 600;
                    min-width: 30px;
                    text-align: center;
                }

                .item-subtotal {
                    display: flex;
                    flex-direction: column;
                    align-items: flex-end;
                }

                .subtotal-label {
                    font-size: 0.75rem;
                    color: var(--color-gray-500);
                }

                .subtotal-value {
                    font-size: 1.125rem;
                    font-weight: 700;
                    color: var(--color-primary);
                }

                .btn-remove {
                    position: absolute;
                    top: var(--spacing-sm);
                    right: var(--spacing-sm);
                    background: var(--color-error);
                    color: white;
                    border: none;
                    border-radius: 50%;
                    width: 30px;
                    height: 30px;
                    font-size: 1.5rem;
                    line-height: 1;
                    cursor: pointer;
                    transition: all var(--transition-fast);
                }

                .btn-remove:hover {
                    transform: scale(1.1);
                }

                .cart-summary {
                    height: fit-content;
                    position: sticky;
                    top: var(--spacing-lg);
                }

                .cart-summary h3 {
                    margin-bottom: var(--spacing-lg);
                    padding-bottom: var(--spacing-md);
                    border-bottom: 2px solid var(--color-gray-200);
                }

                .summary-row {
                    display: flex;
                    justify-content: space-between;
                    margin-bottom: var(--spacing-md);
                }

                .summary-total {
                    font-size: 1.25rem;
                    font-weight: 700;
                    padding-top: var(--spacing-md);
                    margin-top: var(--spacing-md);
                    border-top: 2px solid var(--color-gray-200);
                }

                .checkout-btn {
                    width: 100%;
                    margin: var(--spacing-lg) 0 var(--spacing-md);
                }

                .continue-shopping {
                    width: 100%;
                }

                @media (max-width: 768px) {
                    .cart-content {
                        grid-template-columns: 1fr;
                    }

                    .cart-item {
                        grid-template-columns: 80px 1fr;
                        grid-template-rows: auto auto auto;
                    }

                    .item-quantity,
                    .item-subtotal {
                        grid-column: 2;
                    }
                }
                "#}
            </style>
        </div>
    }
}
