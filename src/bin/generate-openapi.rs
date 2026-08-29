#[cfg(not(feature = "openapi"))]
fn main() {
    println!("openapi feature is required");
}

#[cfg(feature = "openapi")]
fn main() {
    internal::generate();
}

#[cfg(feature = "openapi")]
mod internal {
    extern crate mantle;
    use mantle::web::{
        account::openapi as account, auth::openapi as auth, discord::openapi as discord,
        health::openapi as health, metadata::openapi as metadata,
    };
    use utoipa::{OpenApi, openapi::OpenApi as OpenApiImpl};

    pub fn generate() {
        let doc = collect_docs();

        std::fs::write(
            "openapi.json",
            serde_json::to_string_pretty(&doc).expect("failed to serialize doc"),
        )
        .expect("failed to write openapi.json");
    }

    fn collect_docs() -> OpenApiImpl {
        let mut doc = mantle::web::openapi::ApiDoc::openapi();
        doc.merge(health::ApiDoc::openapi());
        doc.merge(auth::ApiDoc::openapi());
        doc.merge(discord::ApiDoc::openapi());
        doc.merge(metadata::ApiDoc::openapi());
        doc.merge(account::ApiDoc::openapi());
        doc
    }
}
