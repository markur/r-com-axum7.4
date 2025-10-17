// Main App Component with Routing

use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::components::header::Header;
use crate::components::footer::Footer;
use crate::pages::{
    home::HomePage,
    catalog::CatalogPage,
    product::ProductPage,
    cart::CartPage,
    checkout::CheckoutPage,
    not_found::NotFoundPage,
};

#[component]
pub fn App() -> impl IntoView {
    // Provide meta context for SEO
    provide_meta_context();

    view! {
        <Router>
            <div class="app-container">
                // Global meta tags
                <Stylesheet id="leptos" href="/pkg/frontend-leptos.css"/>
                <Title text="R-Com Store - Shop the Latest Products"/>
                <Meta name="description" content="R-Com E-Commerce Platform - Your one-stop shop for quality products"/>
                <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>

                // Header (visible on all pages)
                <Header/>

                // Main content area with routes
                <main class="main-content">
                    <Routes>
                        // Home page
                        <Route path="/" view=HomePage/>

                        // Product catalog
                        <Route path="/catalog" view=CatalogPage/>

                        // Individual product page
                        <Route path="/product/:id" view=ProductPage/>

                        // Shopping cart
                        <Route path="/cart" view=CartPage/>

                        // Checkout flow
                        <Route path="/checkout" view=CheckoutPage/>

                        // 404 Not Found
                        <Route path="/*any" view=NotFoundPage/>
                    </Routes>
                </main>

                // Footer (visible on all pages)
                <Footer/>
            </div>
        </Router>
    }
}
