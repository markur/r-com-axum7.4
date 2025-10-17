// Checkout page with multi-step form

use leptos::*;
use leptos_router::*;
use crate::{
    api::{
        cart::load_cart,
        checkout::create_payment_intent,
    },
    types::{Cart, ShippingAddress},
};

#[component]
pub fn CheckoutPage() -> impl IntoView {
    let navigate = use_navigate();

    // Load cart
    let cart = create_signal(load_cart());

    // Redirect if cart is empty
    create_effect(move |_| {
        if cart.0.get().is_empty() {
            navigate("/cart", Default::default());
        }
    });

    // Form state
    let (street, set_street) = create_signal(String::new());
    let (city, set_city) = create_signal(String::new());
    let (state, set_state) = create_signal(String::new());
    let (zip, set_zip) = create_signal(String::new());
    let (country, set_country) = create_signal("United States".to_string());

    // Processing state
    let (is_processing, set_is_processing) = create_signal(false);
    let (error_message, set_error_message) = create_signal(Option::<String>::None);

    // Handle checkout submission
    let handle_checkout = move |_| {
        set_is_processing(true);
        set_error_message(None);

        let current_cart = cart.0.get();
        let total = current_cart.total();

        spawn_local(async move {
            match create_payment_intent(total).await {
                Ok(response) => {
                    log::info!("Payment intent created: {}", response.client_secret);
                    // TODO: Integrate Stripe Elements here
                    // For now, just show success message
                    set_error_message(Some("Payment processing not yet implemented. Order total: $".to_string() + &format!("{:.2}", total)));
                    set_is_processing(false);
                }
                Err(e) => {
                    log::error!("Payment error: {}", e);
                    set_error_message(Some(format!("Payment failed: {}", e.message)));
                    set_is_processing(false);
                }
            }
        });
    };

    view! {
        <div class="checkout-page container">
            <h1 class="page-title">"Checkout"</h1>

            <div class="checkout-content">
                // Checkout form
                <div class="checkout-form card">
                    <h2>"Shipping Information"</h2>

                    <form on:submit=|e| e.prevent_default()>
                        <div class="form-group">
                            <label>"Street Address"</label>
                            <input
                                type="text"
                                placeholder="123 Main St"
                                value=street
                                on:input=move |ev| set_street(event_target_value(&ev))
                                required
                            />
                        </div>

                        <div class="form-row">
                            <div class="form-group">
                                <label>"City"</label>
                                <input
                                    type="text"
                                    placeholder="New York"
                                    value=city
                                    on:input=move |ev| set_city(event_target_value(&ev))
                                    required
                                />
                            </div>

                            <div class="form-group">
                                <label>"State"</label>
                                <input
                                    type="text"
                                    placeholder="NY"
                                    value=state
                                    on:input=move |ev| set_state(event_target_value(&ev))
                                    required
                                />
                            </div>

                            <div class="form-group">
                                <label>"ZIP Code"</label>
                                <input
                                    type="text"
                                    placeholder="10001"
                                    value=zip
                                    on:input=move |ev| set_zip(event_target_value(&ev))
                                    required
                                />
                            </div>
                        </div>

                        <div class="form-group">
                            <label>"Country"</label>
                            <input
                                type="text"
                                value=country
                                on:input=move |ev| set_country(event_target_value(&ev))
                                required
                            />
                        </div>

                        // Error message
                        <Show when=move || error_message.get().is_some()>
                            <div class="error-message">
                                {move || error_message.get()}
                            </div>
                        </Show>

                        // Submit button
                        <button
                            type="button"
                            class="btn btn-primary btn-lg checkout-btn"
                            on:click=handle_checkout
                            disabled=move || is_processing.get()
                        >
                            <Show
                                when=move || !is_processing.get()
                                fallback=|| view! { <span>"Processing..."</span> }
                            >
                                "Place Order"
                            </Show>
                        </button>
                    </form>
                </div>

                // Order summary
                <div class="order-summary card">
                    <h3>"Order Summary"</h3>

                    // Cart items
                    <div class="summary-items">
                        {move || {
                            cart.0.get().items.into_iter().map(|item| {
                                view! {
                                    <div class="summary-item">
                                        <div class="summary-item-details">
                                            <span class="item-name">{item.product.name}</span>
                                            <span class="item-qty">" × " {item.quantity}</span>
                                        </div>
                                        <span class="item-price">{item.formatted_subtotal()}</span>
                                    </div>
                                }
                            }).collect_view()
                        }}
                    </div>

                    // Totals
                    <div class="summary-totals">
                        <div class="summary-row">
                            <span>"Subtotal:"</span>
                            <span>{move || cart.0.get().formatted_subtotal()}</span>
                        </div>

                        <div class="summary-row">
                            <span>"Tax (8%):"</span>
                            <span>{move || cart.0.get().formatted_tax()}</span>
                        </div>

                        <div class="summary-row summary-total">
                            <span>"Total:"</span>
                            <span>{move || cart.0.get().formatted_total()}</span>
                        </div>
                    </div>
                </div>
            </div>

            <style>
                {r#"
                .checkout-page {
                    padding: var(--spacing-2xl) 0;
                }

                .page-title {
                    text-align: center;
                    margin-bottom: var(--spacing-xl);
                }

                .checkout-content {
                    display: grid;
                    grid-template-columns: 2fr 1fr;
                    gap: var(--spacing-xl);
                }

                .checkout-form h2,
                .order-summary h3 {
                    margin-bottom: var(--spacing-lg);
                    padding-bottom: var(--spacing-md);
                    border-bottom: 2px solid var(--color-gray-200);
                }

                .form-row {
                    display: grid;
                    grid-template-columns: 2fr 1fr 1fr;
                    gap: var(--spacing-md);
                }

                .checkout-btn {
                    width: 100%;
                    margin-top: var(--spacing-lg);
                }

                .error-message {
                    background: var(--color-error);
                    color: white;
                    padding: var(--spacing-md);
                    border-radius: var(--radius-md);
                    margin-top: var(--spacing-md);
                }

                .order-summary {
                    height: fit-content;
                    position: sticky;
                    top: var(--spacing-lg);
                }

                .summary-items {
                    margin-bottom: var(--spacing-lg);
                }

                .summary-item {
                    display: flex;
                    justify-content: space-between;
                    padding: var(--spacing-sm) 0;
                    border-bottom: 1px solid var(--color-gray-200);
                }

                .summary-item-details {
                    display: flex;
                    gap: var(--spacing-xs);
                }

                .item-qty {
                    color: var(--color-gray-500);
                }

                .summary-totals {
                    margin-top: var(--spacing-md);
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

                @media (max-width: 768px) {
                    .checkout-content {
                        grid-template-columns: 1fr;
                    }

                    .form-row {
                        grid-template-columns: 1fr;
                    }
                }
                "#}
            </style>
        </div>
    }
}
