use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, ItemFn, Lit, Meta, Pat, parse_macro_input};

/// Marks one e2e scenario: `#[e2e_test(game = "starboard")]` on
/// `fn name(test: &mut Orchestrator)` expands to the ignored multi-thread
/// tokio test that builds the `Orchestrator` from the environment, replays the
/// body to configure it, runs it with screen recording, and asserts the report.
#[proc_macro_attribute]
pub fn e2e_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let meta = parse_macro_input!(attr as Meta);
    let game = match &meta {
        Meta::NameValue(pair) if pair.path.is_ident("game") => match &pair.value {
            syn::Expr::Lit(lit) => match &lit.lit {
                Lit::Str(text) => text.value(),
                _ => {
                    return quote! {
                        compile_error!("e2e_test: game must be a string literal");
                    }
                    .into();
                }
            },
            _ => {
                return quote! {
                    compile_error!("e2e_test: game must be a string literal");
                }
                .into();
            }
        },
        _ => {
            return quote! {
                compile_error!("#[e2e_test(game = \"...\")] needs a game name");
            }
            .into();
        }
    };
    let func = parse_macro_input!(item as ItemFn);
    let name = &func.sig.ident;
    let binding = match func.sig.inputs.first() {
        Some(FnArg::Typed(typed)) => match &*typed.pat {
            Pat::Ident(named) => &named.ident,
            _ => {
                return quote! {
                    compile_error!("e2e_test: the test fn takes one `&mut Orchestrator` argument");
                }
                .into();
            }
        },
        _ => {
            return quote! {
                compile_error!("e2e_test: the test fn takes one `&mut Orchestrator` argument");
            }
            .into();
        }
    };
    let stmts = &func.block.stmts;
    quote! {
        #[tokio::test(flavor = "multi_thread")]
        #[ignore = "e2e: needs X + fake network + N starboard windows"]
        async fn #name() {
            let mut orchestrator = freenet_libp2p_bevy_plugin::testing::Orchestrator::from_game_and_network(
                #game.to_string(),
                freenet_libp2p_bevy_plugin::testing::Network::from_env(),
            );
            {
                let #binding = &mut orchestrator;
                #(#stmts)*
            }
            let report = crate::harness::run_with_recording(&mut orchestrator).await;
            crate::harness::AssertAll::assert_all(&report);
        }
    }
    .into()
}
