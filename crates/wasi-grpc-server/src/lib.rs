use proc_macro::TokenStream;
use quote::{format_ident, quote};

#[proc_macro_attribute]
pub fn grpc_component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let svr_ty = syn::parse_macro_input!(attr as syn::Path);
    let st = syn::parse_macro_input!(item as syn::ItemStruct);
    let st_name = &st.ident;
    let wasi_implementor = format_ident!("{}GrpcServer", st_name);

    quote!(
        #[doc(hidden)]
        mod __wasi_grpc {
            struct #wasi_implementor;

            ::wasi::http::proxy::export!(#wasi_implementor);

            impl ::wasi::exports::http::incoming_handler::Guest for #wasi_implementor {
                fn handle(request: ::wasi::exports::http::incoming_handler::IncomingRequest, response_out: ::wasi::exports::http::incoming_handler::ResponseOutparam) {
                    let registry = ::wasi_hyperium::poll::Poller::default();
                    let server = super::#svr_ty::new(super::#st_name);
                    let e = ::wasi_hyperium::hyperium1::handle_service_call(server, request, response_out, registry);
                    e.unwrap();
                }
            }
        }

        #st
    )
    .into()
}
