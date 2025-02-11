#![deny(missing_docs)]

//! # Leptos Meta
//!
//! Leptos Meta allows you to modify content in a document’s `<head>` from within components
//! using the [Leptos](https://github.com/gbj/leptos) web framework.
//!
//! Document metadata is updated automatically when running in the browser. For server-side
//! rendering, after the component tree is rendered to HTML, [MetaContext::dehydrate] can generate
//! HTML that should be injected into the `<head>` of the HTML document being rendered.
//!
//! ```
//! use leptos::*;
//! use leptos_meta::*;
//!
//! #[component]
//! fn MyApp(cx: Scope) -> Element {
//!   let (name, set_name) = create_signal(cx, "Alice".to_string());
//!
//!   view! { cx,
//!     <main>
//!       <Title
//!         // reactively sets document.title when `name` changes
//!         text=name
//!         // applies the `formatter` function to the `text` value
//!         formatter=|text| format!("“{text}” is your name")
//!       />
//!       <input
//!         prop:value=name
//!         on:input=move |ev| set_name(event_target_value(&ev))
//!       />
//!     </main>
//!   }
//!
//! }
//!
//! ```

use std::fmt::Debug;

use leptos::{leptos_dom::debug_warn, *};

mod stylesheet;
mod title;
pub use stylesheet::*;
pub use title::*;

/// Contains the current state of meta tags. To access it, you can use [use_head].
///
/// This should generally by provided somewhere in the root of your application using
/// `provide_context(cx, MetaContext::new())`.
#[derive(Debug, Clone, Default)]
pub struct MetaContext {
    pub(crate) title: TitleContext,
    pub(crate) stylesheets: StylesheetContext,
}

/// Returns the current [MetaContext].
///
/// If there is no [MetaContext] in this [Scope](leptos::Scope) or any parent scope, this will
/// create a new [MetaContext] and provide it to the current scope.
///
/// Note that this may cause confusing behavior, e.g., if multiple nested routes independently
/// call `use_head()` but a single [MetaContext] has not been provided at the application root.
/// The best practice is always to `provide_context(cx, MetaContext::new())` early in the application.
pub fn use_head(cx: Scope) -> MetaContext {
    match use_context::<MetaContext>(cx) {
        None => {
            debug_warn!("use_head() is being called with a MetaContext being provided. We'll automatically create and provide one, but if this is being called in a child route it will cause bugs. To be safe, you should provide_context(cx, MetaContext::new()) somewhere in the root of the app.");
            let meta = MetaContext::new();
            provide_context(cx, meta.clone());
            meta
        }
        Some(ctx) => ctx,
    }
}

impl MetaContext {
    /// Creates an empty [MetaContext].
    pub fn new() -> Self {
        Default::default()
    }

    #[cfg(not(any(feature = "csr", feature = "hydrate")))]
    /// Converts the existing metadata tags into HTML that can be injected into the document head.
    ///
    /// This should be called *after* the app’s component tree has been rendered into HTML, so that
    /// components can set meta tags.
    ///
    /// ```
    /// use leptos::*;
    /// use leptos_meta::*;
    ///
    /// # #[cfg(not(any(feature = "csr", feature = "hydrate")))] {
    /// run_scope(create_runtime(), |cx| {
    ///   provide_context(cx, MetaContext::new());
    ///
    ///   let app = view! { cx,
    ///     <main>
    ///       <Title text="my title"/>
    ///       <Stylesheet href="/style.css"/>
    ///       <p>"Some text"</p>
    ///     </main>
    ///   };
    ///
    ///   // `app` contains only the body content w/ hydration stuff, not the meta tags
    ///   assert_eq!(app, r#"<main data-hk="0-0"><!--#--><!--/--><!--#--><!--/--><p>Some text</p></main>"#);
    ///   // `MetaContext::dehydrate()` gives you HTML that should be in the `<head>`
    ///   assert_eq!(use_head(cx).dehydrate(), r#"<title>my title</title><link rel="stylesheet" href="/style.css">"#)
    /// });
    /// # }
    /// ```
    pub fn dehydrate(&self) -> String {
        let mut tags = String::new();

        // Title
        if let Some(title) = self.title.as_string() {
            tags.push_str("<title>");
            tags.push_str(&title);
            tags.push_str("</title>");
        }

        // Stylesheets
        tags.push_str(&self.stylesheets.as_string());

        tags
    }
}

/// Describes a value that is either a static or a reactive string, i.e.,
/// a [String], a [&str], or a reactive `Fn() -> String`.
pub struct TextProp(Box<dyn Fn() -> String>);

impl Debug for TextProp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TextProp").finish()
    }
}

impl From<String> for TextProp {
    fn from(s: String) -> Self {
        TextProp(Box::new(move || s.clone()))
    }
}

impl From<&str> for TextProp {
    fn from(s: &str) -> Self {
        let s = s.to_string();
        TextProp(Box::new(move || s.clone()))
    }
}

impl<F> From<F> for TextProp
where
    F: Fn() -> String + 'static,
{
    fn from(s: F) -> Self {
        TextProp(Box::new(s))
    }
}
