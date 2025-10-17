// Homepage with hero section and featured products

use leptos::*;
use leptos_router::*;
use crate::{
    api::products::fetch_products,
    components::product_card::ProductCard,
    types::Product,
};

#[component]
pub fn HomePage() -> impl IntoView {
    // Fetch featured products (first 6 products)
    let products = create_resource(
        || (),
        |_| async move {
            fetch_products().await.map(|mut p| {
                p.truncate(6);
                p
            })
        },
    );

    view! {
        <div class="home-page">
            // Hero section
            <section class="hero">
                <div class="container">
                    <div class="hero-content">
                        <h1 class="hero-title">"Welcome to R-Com"</h1>
                        <p class="hero-subtitle">
                            "Discover amazing products at unbeatable prices. "
                            "Shop with confidence backed by Rust performance."
                        </p>
                        <div class="hero-buttons">
                            <A href="/catalog" class="btn btn-primary btn-lg">
                                "Shop Now"
                            </A>
                            <a href="#featured" class="btn btn-outline btn-lg">
                                "View Products"
                            </a>
                        </div>
                    </div>
                </div>
            </section>

            // Featured Products section
            <section id="featured" class="featured-products">
                <div class="container">
                    <h2 class="section-title">"Featured Products"</h2>

                    <Suspense fallback=move || view! {
                        <div class="loading">
                            <div class="spinner"></div>
                            <p>"Loading products..."</p>
                        </div>
                    }>
                        {move || {
                            products.get().map(|result| {
                                match result {
                                    Ok(products) if !products.is_empty() => {
                                        view! {
                                            <div class="grid grid-cols-3">
                                                {products
                                                    .into_iter()
                                                    .map(|product| view! { <ProductCard product=product/> })
                                                    .collect_view()
                                                }
                                            </div>
                                            <div class="view-all">
                                                <A href="/catalog" class="btn btn-primary">
                                                    "View All Products"
                                                </A>
                                            </div>
                                        }.into_view()
                                    }
                                    Ok(_) => {
                                        view! {
                                            <div class="empty-state">
                                                <p>"No products available yet."</p>
                                                <p class="text-muted">"Check back soon!"</p>
                                            </div>
                                        }.into_view()
                                    }
                                    Err(e) => {
                                        view! {
                                            <div class="error-state">
                                                <p>"Failed to load products: " {e.message}</p>
                                                <button
                                                    class="btn btn-secondary"
                                                    on:click=move |_| products.refetch()
                                                >
                                                    "Retry"
                                                </button>
                                            </div>
                                        }.into_view()
                                    }
                                }
                            })
                        }}
                    </Suspense>
                </div>
            </section>

            <style>
                {r#"
                .hero {
                    background: linear-gradient(135deg, var(--color-primary) 0%, var(--color-secondary) 100%);
                    color: white;
                    padding: var(--spacing-2xl) 0;
                    text-align: center;
                    min-height: 500px;
                    display: flex;
                    align-items: center;
                }

                .hero-content {
                    max-width: 800px;
                    margin: 0 auto;
                }

                .hero-title {
                    font-size: 3.5rem;
                    font-weight: 900;
                    margin-bottom: var(--spacing-lg);
                    text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.2);
                }

                .hero-subtitle {
                    font-size: 1.25rem;
                    margin-bottom: var(--spacing-xl);
                    opacity: 0.95;
                }

                .hero-buttons {
                    display: flex;
                    gap: var(--spacing-md);
                    justify-content: center;
                    flex-wrap: wrap;
                }

                .featured-products {
                    padding: var(--spacing-2xl) 0;
                }

                .section-title {
                    text-align: center;
                    font-size: 2.5rem;
                    margin-bottom: var(--spacing-xl);
                    color: var(--color-gray-900);
                }

                .loading {
                    text-align: center;
                    padding: var(--spacing-2xl);
                }

                .loading .spinner {
                    margin: 0 auto var(--spacing-md);
                }

                .empty-state,
                .error-state {
                    text-align: center;
                    padding: var(--spacing-2xl);
                    background: var(--color-gray-100);
                    border-radius: var(--radius-lg);
                }

                .text-muted {
                    color: var(--color-gray-500);
                }

                .view-all {
                    text-align: center;
                    margin-top: var(--spacing-xl);
                }

                @media (max-width: 768px) {
                    .hero-title {
                        font-size: 2.5rem;
                    }

                    .hero-subtitle {
                        font-size: 1rem;
                    }

                    .hero {
                        min-height: 400px;
                    }
                }
                "#}
            </style>
        </div>
    }
}
