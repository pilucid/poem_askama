use std::io;

use poem::{get, listener::TcpListener, post, Route, Server};

const LOCALHOST: (&str, u16) = ("0.0.0.0", 8080);

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt().pretty().init();

    let router =
        Route::new().at("/", get(handlers::prompt)).at("/greet", post(handlers::greet));

    let listener = TcpListener::bind(LOCALHOST);

    Server::new(listener).run(router).await
}

mod handlers {
    use poem::{handler, web::Form};
    use tracing::instrument;

    use crate::{
        forms,
        templates::{Greet, Html, Prompt},
    };

    #[allow(unused_braces)]
    #[instrument(ret)]
    #[handler]
    pub fn prompt() -> Html<Prompt> { Html(Prompt) }

    #[instrument(ret)]
    #[handler]
    pub fn greet(Form(form): Form<forms::Greet>) -> Html<Greet> {
        Html(Greet { name: form.name })
    }
}

mod forms {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct Greet {
        pub name: String,
    }
}

mod templates {
    use std::ops::Deref;

    use askama::Template;

    #[derive(Debug)]
    pub struct Html<T>(pub T);

    impl<T> Deref for Html<T> {
        type Target = T;
        fn deref(&self) -> &Self::Target { &self.0 }
    }

    #[derive(Debug, Template)]
    #[template(path = "prompt.html")]
    pub struct Prompt;

    #[derive(Debug, Template)]
    #[template(path = "greet.html")]
    pub struct Greet {
        pub name: String,
    }
}

mod glue {
    use askama::Template;
    use poem::{http::StatusCode, web, IntoResponse, Response};

    use crate::templates;

    impl<T: Template + Send> IntoResponse for templates::Html<T> {
        fn into_response(self) -> Response {
            match self.render() {
                Ok(rendered) => web::Html(rendered).into_response(),
                Err(error) => {
                    let message = format!("Failed to render template: {error}");
                    (StatusCode::INTERNAL_SERVER_ERROR, message).into_response()
                },
            }
        }
    }
}
