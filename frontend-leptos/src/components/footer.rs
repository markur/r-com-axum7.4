// Footer component

use leptos::*;

#[component]
pub fn Footer() -> impl IntoView {
    let current_year = chrono::Utc::now().year();

    view! {
        <footer class="footer">
            <div class="container">
                <div class="footer-content">
                    <div class="footer-section">
                        <h3>"About R-Com"</h3>
                        <p>"Your one-stop shop for quality products. Built with Rust and Leptos."</p>
                    </div>

                    <div class="footer-section">
                        <h3>"Quick Links"</h3>
                        <ul>
                            <li><a href="/catalog">"Shop All Products"</a></li>
                            <li><a href="/cart">"View Cart"</a></li>
                            <li><a href="/checkout">"Checkout"</a></li>
                        </ul>
                    </div>

                    <div class="footer-section">
                        <h3>"Customer Service"</h3>
                        <ul>
                            <li><a href="/contact">"Contact Us"</a></li>
                            <li><a href="/shipping">"Shipping Info"</a></li>
                            <li><a href="/returns">"Returns"</a></li>
                        </ul>
                    </div>
                </div>

                <div class="footer-bottom">
                    <p>"© " {current_year} " R-Com. All rights reserved."</p>
                    <p>"Built with " <span style="color: var(--color-accent);">"♥"</span> " using Rust + Leptos"</p>
                </div>
            </div>

            <style>
                {r#"
                .footer {
                    background: var(--color-gray-900);
                    color: var(--color-gray-300);
                    padding: var(--spacing-2xl) 0 var(--spacing-lg);
                    margin-top: var(--spacing-2xl);
                }

                .footer-content {
                    display: grid;
                    grid-template-columns: repeat(3, 1fr);
                    gap: var(--spacing-xl);
                    margin-bottom: var(--spacing-xl);
                }

                .footer-section h3 {
                    color: white;
                    margin-bottom: var(--spacing-md);
                }

                .footer-section ul {
                    list-style: none;
                    padding: 0;
                }

                .footer-section li {
                    margin-bottom: var(--spacing-sm);
                }

                .footer-section a {
                    color: var(--color-gray-400);
                    transition: color var(--transition-fast);
                }

                .footer-section a:hover {
                    color: var(--color-primary);
                }

                .footer-bottom {
                    border-top: 1px solid var(--color-gray-700);
                    padding-top: var(--spacing-lg);
                    text-align: center;
                }

                .footer-bottom p {
                    margin: var(--spacing-xs) 0;
                    font-size: 0.875rem;
                }

                @media (max-width: 768px) {
                    .footer-content {
                        grid-template-columns: 1fr;
                    }
                }
                "#}
            </style>
        </footer>
    }
}
