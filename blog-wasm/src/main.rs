use crate::client::{BlogClientHttp, BlogClientTrait};
use chrono::{DateTime, Utc};
use derive_more::Display;
use dioxus::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod client;
mod error;

const BASE_URL: &str = "http://127.0.0.1:8080";
const TOKEN_KEY: &str = "blog_token";

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[display("Post {{ id: {}, title: {}, author_id: {} }}", id, title, author_id)]
pub struct Post {
    pub id: Uuid,
    pub author_id: Uuid,
    pub title: String,
    pub content: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
        #[route("/")]
        Home {},
        #[route("/posts")]
        Posts {},
        #[route("/posts/:id")]
        PostDetail { id: Uuid },
        #[route("/create")]
        CreatePost {},
        #[route("/edit/:id")]
        EditPost { id: Uuid },
        #[route("/login")]
        Login {},
        #[route("/register")]
        Register {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let token = use_signal(|| {
        LocalStorage::get::<String>(TOKEN_KEY)
            .ok()
            .filter(|s| !s.is_empty())
    });

    provide_context(token);

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

#[component]
pub fn Hero() -> Element {
    rsx! {
        div {
            id: "hero",
            img { src: HEADER_SVG, id: "header" }
            div { id: "links",
                a { href: "https://dioxuslabs.com/learn/0.7/", "📚 Learn Dioxus" }
                a { href: "https://dioxuslabs.com/awesome", "🚀 Awesome Dioxus" }
                a { href: "https://github.com/dioxus-community/", "📡 Community Libraries" }
                a { href: "https://github.com/DioxusLabs/sdk", "⚙️ Dioxus Development Kit" }
                a { href: "https://marketplace.visualstudio.com/items?itemName=DioxusLabs.dioxus", "💫 VSCode Extension" }
                a { href: "https://discord.gg/XgGxMSkvUM", "👋 Community Discord" }
            }
        }
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        Hero {}
    }
}

#[component]
fn Posts() -> Element {
    let token = use_context::<Signal<Option<String>>>();

    let posts = use_resource(move || async move {
        let mut client =
            BlogClientHttp::connect(BASE_URL)
                .await
                .unwrap_or_else(|_| BlogClientHttp {
                    base_url: BASE_URL.to_string(),
                    token: None,
                });
        client.token = (*token.read()).clone();
        client.list_posts(None, None, None).await
    });

    rsx! {
        div { class: "max-w-6xl mx-auto px-6 py-12",
            h1 { class: "text-4xl font-bold text-white-900 mb-10 text-center", "All Posts" }

            match posts.read().as_ref() {
                Some(Ok(posts)) if !posts.is_empty() => rsx! {
                    div { class: "grid gap-8 md:grid-cols-2 lg:grid-cols-3",
                        for post in posts {
                            article { class: "bg-white rounded-2xl shadow-lg hover:shadow-2xl transition overflow-hidden",
                                Link { to: Route::PostDetail { id: post.id },
                                    div { class: "p-8",
                                        h2 { class: "text-2xl font-bold text-gray-900 mb-3 line-clamp-2", "{post.title}" }
                                        p { class: "text-gray-600 line-clamp-3", "{post.content.chars().take(150).collect::<String>()}..." }
                                        div { class: "mt-6 text-sm text-indigo-600 font-medium", "Read more →" }
                                    }
                                }
                            }
                        }
                    }
                },
                Some(Ok(_)) => rsx! { p { class: "text-center text-gray-500 text-xl", "No posts yet." } },
                Some(Err(e)) => rsx! { div { class: "text-center text-red-600", "Error: {e}" } },
                None => rsx! { div { class: "text-center text-gray-500", "Loading posts..." } },
            }
        }
    }
}

#[component]
fn PostDetail(id: Uuid) -> Element {
    let token = use_context::<Signal<Option<String>>>();
    let navigator = use_navigator();

    let post = use_resource(move || async move {
        let mut client =
            BlogClientHttp::connect(BASE_URL)
                .await
                .unwrap_or_else(|_| BlogClientHttp {
                    base_url: BASE_URL.to_string(),
                    token: None,
                });
        client.token = (*token.read()).clone();
        client.get_post_by_id(id).await
    });

    let on_delete = move |_| {
        if token.read().is_none() {
            navigator.push(Route::Login {});
        };

        spawn(async move {
            let mut client = BlogClientHttp {
                base_url: BASE_URL.to_string(),
                token: None,
            };
            client.token = (*token.read()).clone();
            if client.delete_post(id).await.is_ok() {
                navigator.push(Route::Posts {});
            } else {
                navigator.push(Route::Login {});
            };
        });
    };

    rsx! {
        article { class: "max-w-4xl mx-auto px-6 py-12",
            match post.read().as_ref() {
                Some(Ok(post)) => rsx! {
                    div { class: "bg-white rounded-2xl shadow-xl p-10 md:p-14",
                        h1 { class: "text-4xl md:text-5xl font-bold text-gray-900 mb-8", "{post.title}" }
                        p { class: "text-gray-700 text-lg leading-relaxed whitespace-pre-wrap", "{post.content}" }

                        if token.read().is_some() {
                            div { class: "mt-12 flex gap-4",
                                Link {
                                    to: Route::EditPost { id: post.id },
                                    class: "px-6 py-3 bg-indigo-600 text-white rounded-xl hover:bg-indigo-700 transition shadow-md",
                                    "Edit"
                                }
                                button {
                                    onclick: on_delete,
                                    class: "px-6 py-3 bg-red-600 text-white rounded-xl hover:bg-red-700 transition shadow-md",
                                    "Delete"
                                }
                            }
                        }
                    }
                },
                Some(Err(e)) => rsx! { p { class: "text-center text-red-600 text-xl", "Error: {e}" } },
                None => rsx! { p { class: "text-center text-gray-500 text-xl", "Loading..." } },
            }
        }
    }
}

#[component]
fn CreatePost() -> Element {
    let token_sig = use_context::<Signal<Option<String>>>();
    let navigator = use_navigator();

    if token_sig.read().is_none() {
        navigator.push(Route::Login {});
        return rsx! { "Redirecting to login..." };
    }

    let mut title = use_signal(|| String::new());
    let mut content = use_signal(|| String::new());

    let on_submit = move |_| {
        let title = title.read().clone();
        let content = content.read().clone();
        let token_sig = token_sig.clone();
        let navigator = navigator.clone();

        if title.trim().is_empty() || content.trim().is_empty() {
            return;
        }

        spawn(async move {
            let mut client = BlogClientHttp {
                base_url: BASE_URL.to_string(),
                token: (*token_sig.read()).clone(),
            };
            if let Ok(post) = client.create_post(title, content).await {
                navigator.push(Route::PostDetail { id: post.id });
            }
        });
    };

    rsx! {
        div { class: "max-w-4xl mx-auto px-6 py-12",
            div { class: "bg-white rounded-2xl shadow-xl p-8 md:p-12",
                h1 { class: "text-4xl font-bold text-gray-900 mb-8 text-center", "Create New Post" }

                div { class: "space-y-6",
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-2", "Title" }
                        input {
                            r#type: "text",
                            placeholder: "Enter a catchy title...",
                            value: "{title}",
                            oninput: move |evt| title.set(evt.value()),
                            class: "w-full px-5 py-4 border border-gray-300 rounded-xl shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition text-lg text-black"
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-2", "Content" }
                        textarea {
                            placeholder: "Write your amazing post here... Markdown supported!",
                            value: "{content}",
                            oninput: move |evt| content.set(evt.value()),
                            class: "w-full px-5 py-4 h-96 border border-gray-300 rounded-xl shadow-sm resize-none focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition text-base leading-relaxed font-medium text-black"
                        }
                    }

                    div { class: "flex justify-end gap-4 pt-6",
                        Link {
                            to: Route::Posts {},
                            class: "px-8 py-3.5 border border-gray-300 text-gray-700 rounded-xl hover:bg-gray-50 transition font-medium",
                            "Cancel"
                        }
                        button {
                            onclick: on_submit,
                            class: "px-10 py-3.5 bg-indigo-600 text-white rounded-xl hover:bg-indigo-700 transition shadow-md font-medium text-lg disabled:opacity-50 disabled:cursor-not-allowed",
                            disabled: title.read().trim().is_empty() || content.read().trim().is_empty(),
                            "Create Post"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn EditPost(id: Uuid) -> Element {
    let token_sig = use_context::<Signal<Option<String>>>();
    let navigator = use_navigator();
    let mut post_state = use_signal(|| None::<Result<Post, ()>>);

    // Редирект, если не авторизован
    if token_sig.read().is_none() {
        navigator.push(Route::Login {});
        return rsx! { "Redirecting to login..." };
    }

    // Загружаем пост (исправленный use_future для Dioxus 0.7)
    let post_future = use_future(move || {
        let token = token_sig.read().clone();
        async move {
            let mut client = BlogClientHttp {
                base_url: BASE_URL.to_string(),
                token,
            };
            match client.get_post_by_id(id).await {
                Ok(post) => post_state.set(Some(Ok(post))),
                Err(_) => post_state.set(Some(Err(()))),
            }
        }
    });

    // Локальные сигналы для формы — инициализируются только когда пост загружен
    let mut title = use_signal(String::new);
    let mut content = use_signal(String::new);

    // Синхронизируем форму с загруженным постом (один раз)
    let post_state_clone = post_state.clone();
    if let Some(Ok(post)) = post_state_clone.read().as_ref() {
        if title.read().is_empty() {
            title.set(post.title.clone());
            content.set(post.content.clone());
        }
    }

    let res = match post_state.read().as_ref() {
        Some(Ok(post)) => {
            // Пост загружен — инициализируем сигналы сразу с данными поста
            let mut title = use_signal(|| post.title.clone());
            let mut content = use_signal(|| post.content.clone());

            let on_submit = move |_| {
                let new_title = title.read().clone();
                let new_content = content.read().clone();

                if new_title.trim().is_empty() || new_content.trim().is_empty() {
                    return;
                }

                let token = token_sig.read().clone();
                spawn(async move {
                    let mut client = BlogClientHttp {
                        base_url: BASE_URL.to_string(),
                        token,
                    };
                    if client
                        .update_post(id, Some(new_title), Some(new_content))
                        .await
                        .is_ok()
                    {
                        navigator.push(Route::PostDetail { id });
                    } else {
                        navigator.push(Route::Login {});
                    }
                });
            };

            rsx! {
                div { class: "max-w-4xl mx-auto px-6 py-12",
                    div { class: "bg-white rounded-2xl shadow-xl p-8 md:p-12",
                        h1 { class: "text-4xl font-bold text-gray-900 mb-8 text-center", "Edit Post" }

                        div { class: "space-y-6",
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-2", "Title" }
                                input {
                                    r#type: "text",
                                    value: "{title}",
                                    oninput: move |evt| title.set(evt.value()),
                                    class: "w-full px-5 py-4 border border-gray-300 rounded-xl shadow-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition text-lg text-black"
                                }
                            }

                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-2", "Content" }
                                textarea {
                                    value: "{content}",
                                    oninput: move |evt| content.set(evt.value()),
                                    class: "w-full px-5 py-4 h-96 border border-gray-300 rounded-xl shadow-sm resize-none focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 transition text-base leading-relaxed font-medium text-black"
                                }
                            }

                            div { class: "flex justify-end gap-4 pt-6",
                                Link {
                                    to: Route::PostDetail { id },
                                    class: "px-8 py-3.5 border border-gray-300 text-gray-700 rounded-xl hover:bg-gray-50 transition font-medium",
                                    "Cancel"
                                }
                                button {
                                    onclick: on_submit,
                                    disabled: title.read().trim().is_empty() || content.read().trim().is_empty(),
                                    class: "px-10 py-3.5 bg-indigo-600 text-white rounded-xl hover:bg-indigo-700 transition shadow-md font-medium text-lg disabled:opacity-50 disabled:cursor-not-allowed",
                                    "Save Changes"
                                }
                            }
                        }
                    }
                }
            }
        }
        Some(&Err(_)) => rsx! {
            div { class: "max-w-4xl mx-auto px-6 py-12 text-center",
                p { class: "text-red-600 text-xl mb-6", "Post not found or access denied" }
                Link {
                    to: Route::Posts {},
                    class: "inline-block px-8 py-3 bg-indigo-600 text-white rounded-xl hover:bg-indigo-700 transition",
                    "Back to posts"
                }
            }
        },
        None => rsx! {
            div { class: "flex justify-center items-center min-h-screen",
                div { class: "animate-spin rounded-full h-16 w-16 border-4 border-indigo-600 border-t-transparent" }
            }
        },
    };
    res
}

#[component]
fn Login() -> Element {
    let token_sig = use_context::<Signal<Option<String>>>();
    let navigator = use_navigator();

    let mut username = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());

    let on_submit = move |evt: FormEvent| {
        evt.prevent_default();
        let username = username.read().clone();
        let password = password.read().clone();
        let mut token_sig = token_sig.clone();
        let navigator = navigator.clone();
        spawn(async move {
            let mut client = BlogClientHttp {
                base_url: BASE_URL.to_string(),
                token: None,
            };
            if client.login(username, password).await.is_ok() {
                token_sig.set(LocalStorage::get::<String>(TOKEN_KEY).ok());
                navigator.push(Route::Home {});
            }
        });
    };

    rsx! {
        div {
            class: "min-h-screen flex items-center justify-center bg-gray-50 px-4",

            // Карточка формы
            div {
                class: "w-full max-w-md bg-white rounded-2xl shadow-xl p-8 md:p-10",

                h1 {
                    class: "text-3xl font-bold text-center text-gray-900 mb-8",
                    "Login to Blog"
                }

                form {
                    onsubmit: on_submit,
                    class: "space-y-6",

                    div {
                        input {
                            class: "w-full px-5 py-4 text-lg border border-gray-300 rounded-xl shadow-sm
                                 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500
                                 transition placeholder-gray-500 text-black",
                            r#type: "text",
                            placeholder: "Username",
                            value: "{username}",
                            oninput: move |evt| username.set(evt.value()),
                        }
                    }

                    div {
                        input {
                            class: "w-full px-5 py-4 text-lg border border-gray-300 rounded-xl shadow-sm
                                 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500
                                 transition placeholder-gray-500 text-black",
                            r#type: "password",
                            placeholder: "Password",
                            value: "{password}",
                            oninput: move |evt| password.set(evt.value()),
                        }
                    }

                    button {
                        class: "w-full py-4 bg-blue-600 hover:bg-blue-700 text-white font-semibold
                             rounded-xl shadow-md transition transform hover:-translate-y-0.5
                             active:translate-y-0 disabled:opacity-50 disabled:cursor-not-allowed",
                        r#type: "submit",
                        disabled: username.read().trim().is_empty() || password.read().len() < 3,
                        "Login"
                    }

                    // Опционально — ссылка на регистрацию
                    div { class: "text-center mt-6",
                        Link {
                            to: Route::Register {},
                            class: "text-blue-600 hover:text-blue-800 font-medium transition",
                            "Don't have an account? Sign up"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Register() -> Element {
    let token_sig = use_context::<Signal<Option<String>>>();
    let navigator = use_navigator();

    let mut username = use_signal(|| String::new());
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());

    let on_submit = move |evt: FormEvent| {
        evt.prevent_default();
        let username = username.read().clone();
        let email = email.read().clone();
        let password = password.read().clone();
        let mut token_sig = token_sig.clone();
        let navigator = navigator.clone();
        spawn(async move {
            let mut client = BlogClientHttp {
                base_url: BASE_URL.to_string(),
                token: None,
            };
            if client.register(username, email, password).await.is_ok() {
                token_sig.set(LocalStorage::get::<String>(TOKEN_KEY).ok());
                navigator.push(Route::Home {});
            }
        });
    };

    rsx! {
        div {
            class: "min-h-screen flex items-center justify-center bg-gray-50 px-4 py-12",

            // Карточка формы регистрации
            div {
                class: "w-full max-w-md bg-white rounded-2xl shadow-xl p-8 md:p-10",

                h1 {
                    class: "text-3xl font-bold text-center text-gray-900 mb-8",
                    "Create Account"
                }

                form {
                    onsubmit: on_submit,
                    class: "space-y-6",

                    // Username
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "Username" }
                        input {
                            class: "w-full px-5 py-4 text-lg border border-gray-300 rounded-xl shadow-sm
                                 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500
                                 transition placeholder-gray-500 text-black",
                            r#type: "text",
                            placeholder: "Choose a username",
                            value: "{username}",
                            oninput: move |evt| username.set(evt.value().trim().to_string()),
                            required: true,
                        }
                    }

                    // Email
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "Email" }
                        input {
                            class: "w-full px-5 py-4 text-lg border border-gray-300 rounded-xl shadow-sm
                                 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500
                                 transition placeholder-gray-500 text-black",
                            r#type: "email",
                            placeholder: "you@example.com",
                            value: "{email}",
                            oninput: move |evt| email.set(evt.value().trim().to_string()),
                            required: true,
                        }
                    }

                    // Password
                    div {
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "Password" }
                        input {
                            class: "w-full px-5 py-4 text-lg border border-gray-300 rounded-xl shadow-sm
                                 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500
                                 transition placeholder-gray-500 text-black",
                            r#type: "password",
                            placeholder: "At least 6 characters",
                            value: "{password}",
                            oninput: move |evt| password.set(evt.value()),
                            minlength: 6,
                            required: true,
                        }
                    }

                    // Кнопка регистрации
                    button {
                        class: "w-full py-4 bg-blue-600 hover:bg-blue-700 text-white font-semibold text-lg
                             rounded-xl shadow-md transition transform hover:-translate-y-0.5
                             active:translate-y-0 disabled:opacity-50 disabled:cursor-not-allowed",
                        r#type: "submit",
                        disabled: {
                            let u = username.read();
                            let e = email.read();
                            let p = password.read();
                            u.trim().is_empty() || e.trim().is_empty() || p.len() < 6
                        },
                        "Create Account"
                    }

                    // Ссылка на логин
                    div { class: "text-center pt-4",
                        Link {
                            to: Route::Login {},
                            class: "text-blue-600 hover:text-blue-800 font-medium transition",
                            "Already have an account? Log in"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Navbar() -> Element {
    let mut token = use_context::<Signal<Option<String>>>();
    let navigator = use_navigator();

    let is_logged_in = token.read().is_some();

    let on_logout = move |_| {
        LocalStorage::delete(TOKEN_KEY);
        token.set(None);
        navigator.push(Route::Home {});
    };

    rsx! {
        nav { class: "bg-white/80 backdrop-blur-md border-b border-gray-200 sticky top-0 z-50 shadow-sm",
            div { class: "max-w-7xl mx-auto px-6 py-4 flex justify-between items-center",
                div { class: "flex items-center space-x-8",
                    Link { to: Route::Home {}, class: "text-2xl font-bold text-indigo-600 hover:text-indigo-700 transition", "MyBlog" }
                    div { class: "hidden md:flex space-x-6",
                        Link { to: Route::Posts {}, class: "text-gray-700 hover:text-indigo-600 font-medium transition", "Posts" }
                        if is_logged_in {
                            Link { to: Route::CreatePost {}, class: "text-gray-700 hover:text-indigo-600 font-medium transition", "New Post" }
                        }
                    }
                }

                div { class: "flex items-center space-x-4",
                    if is_logged_in {
                        button {
                            onclick: on_logout,
                            class: "px-5 py-2.5 bg-red-500 text-white rounded-xl hover:bg-red-600 transition shadow-md",
                            "Logout"
                        }
                    } else {
                        Link {
                            to: Route::Login {},
                            class: "px-5 py-2.5 bg-indigo-600 text-white rounded-xl hover:bg-indigo-700 transition shadow-md",
                            "Login"
                        }
                        Link {
                            to: Route::Register {},
                            class: "px-5 py-2.5 border border-indigo-600 text-indigo-600 rounded-xl hover:bg-indigo-50 transition",
                            "Register"
                        }
                    }
                }
            }
        }
        Outlet::<Route> {}
    }
}
