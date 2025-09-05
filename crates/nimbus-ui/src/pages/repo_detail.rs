use leptos::*;
use leptos_router::*;

#[component]
pub fn RepoDetail() -> impl IntoView {
    let params = use_params_map();
    let name = move || params.with(|p| p.get("name").cloned().unwrap_or_default());

    view! {
        <div>
            <div class="mb-6">
                <h1 class="text-3xl font-bold">{name}</h1>
                <p class="text-gray-600 mt-2">"A great repository with amazing code"</p>
            </div>

            <div class="flex space-x-4 mb-6 border-b">
                <TabButton label="Code" active=true/>
                <TabButton label="Pull Requests" active=false/>
                <TabButton label="Actions" active=false/>
                <TabButton label="Settings" active=false/>
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-4 gap-6">
                <div class="lg:col-span-3">
                    <FileExplorer/>
                </div>
                <div>
                    <RepoSidebar/>
                </div>
            </div>
        </div>
    }
}

#[component]
fn TabButton(label: &'static str, active: bool) -> impl IntoView {
    let class = if active {
        "px-4 py-2 border-b-2 border-blue-600 text-blue-600 font-medium"
    } else {
        "px-4 py-2 text-gray-600 hover:text-gray-900"
    };

    view! {
        <button class=class>{label}</button>
    }
}

#[component]
fn FileExplorer() -> impl IntoView {
    view! {
        <div class="bg-white rounded-lg shadow">
            <div class="p-4 border-b bg-gray-50">
                <div class="flex items-center justify-between">
                    <div class="flex items-center space-x-2">
                        <button class="px-3 py-1 bg-gray-200 rounded hover:bg-gray-300">
                            "main ▼"
                        </button>
                        <span class="text-sm text-gray-600">"12 branches"</span>
                    </div>
                    <div class="flex space-x-2">
                        <button class="px-3 py-1 bg-green-600 text-white rounded hover:bg-green-700">
                            "Clone"
                        </button>
                    </div>
                </div>
            </div>

            <div class="p-4">
                <div class="space-y-2">
                    <FileRow name="src/" is_dir=true/>
                    <FileRow name="Cargo.toml" is_dir=false/>
                    <FileRow name="README.md" is_dir=false/>
                    <FileRow name=".gitignore" is_dir=false/>
                </div>
            </div>
        </div>
    }
}

#[component]
fn FileRow(name: &'static str, is_dir: bool) -> impl IntoView {
    let icon = if is_dir { "📁" } else { "📄" };

    view! {
        <div class="flex items-center py-2 px-2 hover:bg-gray-50 rounded cursor-pointer">
            <span class="mr-2">{icon}</span>
            <span class="font-mono text-sm">{name}</span>
        </div>
    }
}

#[component]
fn RepoSidebar() -> impl IntoView {
    view! {
        <div class="space-y-4">
            <div class="bg-white rounded-lg shadow p-4">
                <h3 class="font-semibold mb-3">"About"</h3>
                <p class="text-sm text-gray-600 mb-3">
                    "Cloud-native git platform with plugin architecture"
                </p>
                <div class="space-y-2 text-sm">
                    <div>"⭐ 42 stars"</div>
                    <div>"🍴 5 forks"</div>
                    <div>"📝 MIT License"</div>
                </div>
            </div>

            <div class="bg-white rounded-lg shadow p-4">
                <h3 class="font-semibold mb-3">"Languages"</h3>
                <div class="space-y-2">
                    <div class="flex items-center justify-between text-sm">
                        <span>"Rust"</span>
                        <span class="text-gray-500">"89.2%"</span>
                    </div>
                    <div class="flex items-center justify-between text-sm">
                        <span>"HTML"</span>
                        <span class="text-gray-500">"7.1%"</span>
                    </div>
                    <div class="flex items-center justify-between text-sm">
                        <span>"CSS"</span>
                        <span class="text-gray-500">"3.7%"</span>
                    </div>
                </div>
            </div>
        </div>
    }
}
