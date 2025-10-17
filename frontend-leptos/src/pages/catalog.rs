// Product catalog/listing page

use leptos::*;
use crate::{
    api::products::fetch_products,
    components::product_card::ProductCard,
    types::{Product, product::ProductSortOrder},
};

#[component]
pub fn CatalogPage() -> impl IntoView {
    // Fetch all products
    let products = create_resource(
        || (),
        |_| async move { fetch_products().await },
    );

    // Sort order state
    let (sort_order, set_sort_order) = create_signal(ProductSortOrder::Newest);

    // Search/filter state
    let (search_query, set_search_query) = create_signal(String::new());

    // Filtered and sorted products
    let filtered_products = move || {
        products.get().and_then(|result| {
            result.ok().map(|mut prods| {
                // Filter by search query
                let query = search_query.get().to_lowercase();
                if !query.is_empty() {
                    prods.retain(|p| {
                        p.name.to_lowercase().contains(&query)
                            || p.description.as_ref().map_or(false, |d| d.to_lowercase().contains(&query))
                    });
                }

                // Sort products
                match sort_order.get() {
                    ProductSortOrder::NameAsc => prods.sort_by(|a, b| a.name.cmp(&b.name)),
                    ProductSortOrder::NameDesc => prods.sort_by(|a, b| b.name.cmp(&a.name)),
                    ProductSortOrder::PriceAsc => prods.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap()),
                    ProductSortOrder::PriceDesc => prods.sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap()),
                    ProductSortOrder::Newest => prods.reverse(),
                }

                prods
            })
        })
    };

    view! {
        <div class="catalog-page container">
            <h1 class="page-title">"Shop All Products"</h1>

            // Filters and controls
            <div class="catalog-controls">
                // Search bar
                <div class="search-bar">
                    <input
                        type="text"
                        placeholder="Search products..."
                        value=search_query
                        on:input=move |ev| set_search_query(event_target_value(&ev))
                    />
                </div>

                // Sort dropdown
                <div class="sort-controls">
                    <label>"Sort by:"</label>
                    <select on:change=move |ev| {
                        let value = event_target_value(&ev);
                        let order = match value.as_str() {
                            "name_asc" => ProductSortOrder::NameAsc,
                            "name_desc" => ProductSortOrder::NameDesc,
                            "price_asc" => ProductSortOrder::PriceAsc,
                            "price_desc" => ProductSortOrder::PriceDesc,
                            _ => ProductSortOrder::Newest,
                        };
                        set_sort_order(order);
                    }>
                        <option value="newest">"Newest First"</option>
                        <option value="name_asc">"Name (A-Z)"</option>
                        <option value="name_desc">"Name (Z-A)"</option>
                        <option value="price_asc">"Price (Low to High)"</option>
                        <option value="price_desc">"Price (High to Low)"</option>
                    </select>
                </div>
            </div>

            // Products grid
            <Suspense fallback=move || view! {
                <div class="loading">
                    <div class="spinner"></div>
                    <p>"Loading products..."</p>
                </div>
            }>
                {move || {
                    filtered_products().map(|prods| {
                        if prods.is_empty() {
                            view! {
                                <div class="empty-state">
                                    <p>"No products found."</p>
                                    <Show when=move || !search_query.get().is_empty()>
                                        <button
                                            class="btn btn-secondary"
                                            on:click=move |_| set_search_query(String::new())
                                        >
                                            "Clear Search"
                                        </button>
                                    </Show>
                                </div>
                            }.into_view()
                        } else {
                            view! {
                                <div>
                                    <p class="results-count">
                                        "Showing " {prods.len()} " product(s)"
                                    </p>
                                    <div class="grid grid-cols-4">
                                        {prods
                                            .into_iter()
                                            .map(|product| view! { <ProductCard product=product/> })
                                            .collect_view()
                                        }
                                    </div>
                                </div>
                            }.into_view()
                        }
                    })
                }}
            </Suspense>

            <style>
                {r#"
                .catalog-page {
                    padding: var(--spacing-2xl) 0;
                }

                .page-title {
                    text-align: center;
                    margin-bottom: var(--spacing-xl);
                }

                .catalog-controls {
                    display: flex;
                    gap: var(--spacing-lg);
                    margin-bottom: var(--spacing-xl);
                    align-items: flex-end;
                }

                .search-bar {
                    flex: 1;
                }

                .sort-controls {
                    display: flex;
                    gap: var(--spacing-sm);
                    align-items: center;
                }

                .sort-controls label {
                    margin: 0;
                    font-weight: 500;
                }

                .sort-controls select {
                    min-width: 200px;
                }

                .results-count {
                    margin-bottom: var(--spacing-md);
                    color: var(--color-gray-600);
                    font-size: 0.875rem;
                }

                .loading,
                .empty-state {
                    text-align: center;
                    padding: var(--spacing-2xl);
                }

                @media (max-width: 768px) {
                    .catalog-controls {
                        flex-direction: column;
                        align-items: stretch;
                    }

                    .sort-controls select {
                        min-width: auto;
                        width: 100%;
                    }
                }
                "#}
            </style>
        </div>
    }
}
