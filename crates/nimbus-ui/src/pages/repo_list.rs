use leptos::*;
use leptos_router::*;

#[derive(Clone, Debug)]
struct Repo {
    name: String,
    description: String,
    language: String,
    stars: u32,
    updated: String,
}

#[component]
pub fn RepoList() -> impl IntoView {
    // Mock data - will be replaced with API calls
    let repos = vec![
        Repo {
            name: "nimbus-git".to_string(),
            description: "Cloud-native git platform with plugin architecture".to_string(),
            language: "Rust".to_string(),
            stars: 42,
            updated: "2 hours ago".to_string(),
        },
        Repo {
            name: "awesome-project".to_string(),
            description: "Something awesome built with Rust".to_string(),
            language: "Rust".to_string(),
            stars: 128,
            updated: "1 day ago".to_string(),
        },
        Repo {
            name: "web-experiments".to_string(),
            description: "WASM and Leptos experiments".to_string(),
            language: "Rust".to_string(),
            stars: 15,
            updated: "3 days ago".to_string(),
        },
    ];

    view! {
        <div>
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-3xl font-bold">"Repositories"</h1>
                <button class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700">
                    "New Repository"
                </button>
            </div>

            <div class="bg-white rounded-lg shadow">
                <div class="p-4 border-b">
                    <input
                        type="text"
                        placeholder="Search repositories..."
                        class="w-full px-3 py-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                </div>

                <div class="divide-y">
                    {repos
                        .into_iter()
                        .map(|repo| {
                            view! {
                                <RepoItem repo=repo/>
                            }
                        })
                        .collect_view()}
                </div>
            </div>
        </div>
    }
}

#[component]
fn RepoItem(repo: Repo) -> impl IntoView {
    let owner = "owner"; // In single-owner model, this is always the instance owner
    
    view! {
        <div class="p-4 hover:bg-gray-50">
            <div class="flex items-start justify-between">
                <div class="flex-1">
                    <A
                        href=format!("/repos/{}/{}", owner, repo.name)
                        class="text-lg font-semibold text-blue-600 hover:underline"
                    >
                        {repo.name.clone()}
                    </A>
                    <p class="text-gray-600 mt-1">{repo.description}</p>
                    <div class="flex items-center space-x-4 mt-3 text-sm text-gray-500">
                        <span class="flex items-center">
                            <span class="w-3 h-3 bg-orange-400 rounded-full mr-1"></span>
                            {repo.language}
                        </span>
                        <span>"⭐ " {repo.stars}</span>
                        <span>"Updated " {repo.updated}</span>
                    </div>
                </div>
                <div class="flex space-x-2 ml-4">
                    <button class="text-gray-600 hover:text-gray-900">
                        "⚙️"
                    </button>
                </div>
            </div>
        </div>
    }
}