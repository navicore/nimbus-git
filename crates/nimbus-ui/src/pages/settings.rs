use leptos::*;

#[component]
pub fn Settings() -> impl IntoView {
    view! {
        <div>
            <h1 class="text-3xl font-bold mb-8">"Settings"</h1>

            <div class="space-y-6">
                <SettingsSection title="Instance Configuration">
                    <SettingRow
                        label="Instance Domain"
                        value="code.example.com"
                        editable=true
                    />
                    <SettingRow
                        label="Owner Email"
                        value="owner@example.com"
                        editable=true
                    />
                    <SettingRow
                        label="Instance Name"
                        value="My Nimbus Instance"
                        editable=true
                    />
                </SettingsSection>

                <SettingsSection title="Collaborators">
                    <div class="space-y-3">
                        <CollaboratorRow name="Alice Developer" email="alice@example.com"/>
                        <CollaboratorRow name="Bob Reviewer" email="bob@example.com"/>
                        <button class="text-blue-600 hover:text-blue-800">
                            "+ Add Collaborator"
                        </button>
                    </div>
                </SettingsSection>

                <SettingsSection title="Plugins">
                    <PluginRow
                        name="GitHub Actions Runner"
                        status="Active"
                        description="Run GitHub Actions workflows"
                    />
                    <PluginRow
                        name="Claude AI Assistant"
                        status="Active"
                        description="AI-powered code review and assistance"
                    />
                    <PluginRow
                        name="Slack Notifications"
                        status="Inactive"
                        description="Send notifications to Slack"
                    />
                </SettingsSection>

                <SettingsSection title="API Keys">
                    <div class="space-y-3">
                        <ApiKeyRow name="CI/CD Token" last_used="2 days ago"/>
                        <ApiKeyRow name="MCP Integration" last_used="1 hour ago"/>
                        <button class="text-blue-600 hover:text-blue-800">
                            "+ Generate New API Key"
                        </button>
                    </div>
                </SettingsSection>
            </div>
        </div>
    }
}

#[component]
fn SettingsSection(title: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class="bg-white rounded-lg shadow p-6">
            <h2 class="text-xl font-semibold mb-4">{title}</h2>
            <div class="space-y-4">
                {children()}
            </div>
        </div>
    }
}

#[component]
fn SettingRow(label: &'static str, value: &'static str, editable: bool) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between py-2">
            <span class="text-gray-700">{label}</span>
            <div class="flex items-center space-x-2">
                <span class="text-gray-900">{value}</span>
                {editable.then(|| view! {
                    <button class="text-gray-500 hover:text-gray-700">
                        "✏️"
                    </button>
                })}
            </div>
        </div>
    }
}

#[component]
fn CollaboratorRow(name: &'static str, email: &'static str) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between py-2 border-b">
            <div>
                <div class="font-medium">{name}</div>
                <div class="text-sm text-gray-600">{email}</div>
            </div>
            <button class="text-red-600 hover:text-red-800">
                "Remove"
            </button>
        </div>
    }
}

#[component]
fn PluginRow(name: &'static str, status: &'static str, description: &'static str) -> impl IntoView {
    let status_class = if status == "Active" { "text-green-600" } else { "text-gray-500" };

    view! {
        <div class="flex items-center justify-between py-3 border-b last:border-0">
            <div>
                <div class="font-medium">{name}</div>
                <div class="text-sm text-gray-600">{description}</div>
            </div>
            <div class="flex items-center space-x-3">
                <span class=format!("text-sm {}", status_class)>{status}</span>
                <button class="text-blue-600 hover:text-blue-800">
                    "Configure"
                </button>
            </div>
        </div>
    }
}

#[component]
fn ApiKeyRow(name: &'static str, last_used: &'static str) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between py-2 border-b">
            <div>
                <div class="font-medium font-mono text-sm">{name}</div>
                <div class="text-sm text-gray-600">"Last used: " {last_used}</div>
            </div>
            <button class="text-red-600 hover:text-red-800">
                "Revoke"
            </button>
        </div>
    }
}
