// Header component with navigation and cart indicator

use leptos::*;
use leptos_router::*;
use crate::api::cart::load_cart;

#[component]
pub fn Header() -> impl IntoView {
    // Load cart to show item count
    let cart = create_rw_signal(load_cart());
    let cart_count = move || cart.get().total_items();

    view! {
        <header class="header">
            <div class="container">
                <nav class="nav">
                    // Logo and brand
                    <div class="nav-brand">
                        <A href="/" class="logo">
                            <h1>"R-Com"</h1>
                        </A>
                    </div>

                    // Navigation links
                    <div class="nav-links">
                        <A href="/" class="nav-link">"Home"</A>
                        <A href="/catalog" class="nav-link">"Shop"</A>
                        <A href="/cart" class="nav-link cart-link">
                            "Cart "
                            <Show
                                when=move || cart_count() > 0
                                fallback=|| view! { <span></span> }
                            >
                                <span class="badge badge-primary">
                                    {cart_count}
                                </span>
                            </Show>
                        </A>
                    </div>
                </nav>
            </div>

            <style>
                {r#"
                .header {
                    background: linear-gradient(135deg, var(--color-primary) 0%, var(--color-secondary) 100%);
                    color: white;
                    padding: var(--spacing-md) 0;
                    box-shadow: var(--shadow-lg);
                    position: sticky;
                    top: 0;
                    z-index: 1000;
                }

                .nav {
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                }

                .nav-brand .logo {
                    color: white;
                    text-decoration: none;
                }

                .nav-brand h1 {
                    margin: 0;
                    font-size: 1.75rem;
                    font-weight: 700;
                }

                .nav-links {
                    display: flex;
                    gap: var(--spacing-lg);
                    align-items: center;
                }

                .nav-link {
                    color: white;
                    text-decoration: none;
                    font-weight: 500;
                    transition: all var(--transition-fast);
                    padding: var(--spacing-sm) var(--spacing-md);
                    border-radius: var(--radius-md);
                }

                .nav-link:hover {
                    background: rgba(255, 255, 255, 0.1);
                    text-decoration: none;
                }

                .cart-link {
                    position: relative;
                    display: flex;
                    align-items: center;
                    gap: var(--spacing-xs);
                }

                @media (max-width: 768px) {
                    .nav-brand h1 {
                        font-size: 1.25rem;
                    }

                    .nav-links {
                        gap: var(--spacing-sm);
                    }

                    .nav-link {
                        font-size: 0.875rem;
                        padding: var(--spacing-xs) var(--spacing-sm);
                    }
                }
                "#}
            </style>
        </header>
    }
}
