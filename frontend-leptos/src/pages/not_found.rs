// 404 Not Found page

use leptos::*;
use leptos_router::*;

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <div class="not-found-page container">
            <div class="not-found-content">
                <h1 class="error-code">"404"</h1>
                <h2>"Page Not Found"</h2>
                <p>"Sorry, the page you're looking for doesn't exist."</p>
                <A href="/" class="btn btn-primary">"Go Home"</A>
            </div>

            <style>
                {r#"
                .not-found-page {
                    min-height: 50vh;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    text-align: center;
                }

                .not-found-content {
                    max-width: 500px;
                }

                .error-code {
                    font-size: 6rem;
                    font-weight: 900;
                    color: var(--color-primary);
                    margin: 0;
                    line-height: 1;
                }

                .not-found-content h2 {
                    font-size: 2rem;
                    margin: var(--spacing-md) 0;
                }

                .not-found-content p {
                    color: var(--color-gray-600);
                    margin-bottom: var(--spacing-xl);
                }
                "#}
            </style>
        </div>
    }
}
