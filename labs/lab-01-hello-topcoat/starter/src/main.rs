use topcoat::{
    Result,
    router::{Router, RouterBuilderDiscoverExt, page},
    view::{component, view},
};

#[tokio::main]
async fn main() {
    topcoat::start(Router::builder().discover().build()).await.unwrap();
}

#[page("/")]
async fn home() -> Result {
    view! {
        <!DOCTYPE html>
        <html>
            <head>
                <title>"Hello world"</title>
                // TODO(lab-01): add topcoat::dev::script() here so the page hot-reloads under `topcoat dev`
            </head>
            <body>
                // TODO(lab-01): replace this with the `hello` component, passing your name
                <h1>"Hello, World!"</h1>
            </body>
        </html>
    }
}

// TODO(lab-01): write a `hello` component that takes `name: &str` and renders <h1>Hello, {name}!</h1>
