use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(FlashrustDatabase)]
pub fn database(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).expect("Could not parse database macro input");

    let name = &ast.ident;
    let generated = quote! {
        use database::traits::BaseDatabase;

        #[async_trait::async_trait]
        impl BaseDatabase for #name {

            #[cfg(feature = "sqlite")]
            type Db = sqlx::sqlite::Sqlite;

            #[cfg(not(feature = "sqlite"))]
            type Db = sqlx::postgres::Postgres;


            #[cfg(feature = "sqlite")]
            async fn connect(connection_string: &str) -> Result<sqlx::Pool<Self::Db>, sqlx::Error> {
                Ok(
                    sqlx::sqlite::SqlitePoolOptions::new()
                    .connect(connection_string)
                    .await?
                )
            }

            #[cfg(not(feature = "sqlite"))]
            async fn connect(connection_string: &str) -> Result<sqlx::Pool<Self::Db>, sqlx::Error> {
                Ok(
                    sqlx::postgres::PgPoolOptions::new()
                    .connect(connection_string)
                    .await?
                )
            }
        }
    };

    generated.into()
}
