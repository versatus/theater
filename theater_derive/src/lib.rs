use proc_macro::{self, TokenStream};
use quote::quote;
use syn;

#[proc_macro_derive(Actor)]
pub fn actor_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_actor_macro(&ast)
}

// TODO: work on the derive API
fn impl_actor_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        #[async_trait]
        impl Actor for #name {
        }
    };

    gen.into()
}

#[cfg(test)]
mod tests {}
