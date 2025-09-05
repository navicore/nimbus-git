use leptos::*;

#[component]
pub fn Dashboard() -> impl IntoView {
    // Mock data for now
    let recent_activity = vec![
        ("Push to main", "nimbus-git", "2 hours ago"),
        ("Merged PR #42", "awesome-project", "5 hours ago"),
        ("Created tag v1.0.0", "rust-lib", "1 day ago"),
    ];

    view! {
        <div>
            <h1 class="text-3xl font-bold mb-8">"Dashboard"</h1>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
                <StatCard title="Repositories" value="12" icon="📦"/>
                <StatCard title="Open PRs" value="3" icon="🔄"/>
                <StatCard title="CI Runs Today" value="27" icon="🚀"/>
            </div>

            <div class="bg-white rounded-lg shadow p-6">
                <h2 class="text-xl font-semibold mb-4">"Recent Activity"</h2>
                <div class="space-y-3">
                    {recent_activity
                        .into_iter()
                        .map(|(action, repo, time)| {
                            view! {
                                <div class="flex items-center justify-between py-2 border-b last:border-0">
                                    <div>
                                        <span class="font-medium">{action}</span>
                                        " in "
                                        <span class="text-blue-600">{repo}</span>
                                    </div>
                                    <span class="text-sm text-gray-500">{time}</span>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
            </div>
        </div>
    }
}

#[component]
fn StatCard(title: &'static str, value: &'static str, icon: &'static str) -> impl IntoView {
    view! {
        <div class="bg-white rounded-lg shadow p-6">
            <div class="flex items-center justify-between">
                <div>
                    <p class="text-sm text-gray-600">{title}</p>
                    <p class="text-2xl font-bold">{value}</p>
                </div>
                <span class="text-3xl">{icon}</span>
            </div>
        </div>
    }
}
