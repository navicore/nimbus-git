use leptos::*;
use leptos_meta::*;
use leptos_router::*;

mod components;
mod pages;

use pages::{Dashboard, RepoDetail, RepoList, Settings};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/nimbus-ui.css"/>
        <Title text="Nimbus Git"/>
        <Meta charset="utf-8"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1"/>

        <Router>
            <Nav/>
            <main class="container mx-auto px-4 py-8">
                <Routes>
                    <Route path="/" view=Dashboard/>
                    <Route path="/repos" view=RepoList/>
                    <Route path="/repos/:owner/:name" view=RepoDetail/>
                    <Route path="/settings" view=Settings/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn Nav() -> impl IntoView {
    view! {
        <nav class="bg-gray-900 text-white p-4">
            <div class="container mx-auto flex items-center justify-between">
                <div class="flex items-center space-x-8">
                    <A href="/" class="text-xl font-bold hover:text-gray-300">
                        "Nimbus"
                    </A>
                    <div class="flex space-x-4">
                        <A href="/repos" class="hover:text-gray-300">
                            "Repositories"
                        </A>
                        <A href="/settings" class="hover:text-gray-300">
                            "Settings"
                        </A>
                    </div>
                </div>
                <div class="flex items-center space-x-4">
                    <span class="text-sm text-gray-400">
                        "Single Owner Instance"
                    </span>
                </div>
            </div>
        </nav>
    }
}