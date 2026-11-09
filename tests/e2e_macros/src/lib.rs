use proc_macro::TokenStream;
use quote::quote;
use syn::{FnArg, ItemFn, Pat, parse_macro_input};

/// Marks one e2e scenario: `#[e2e_test(game = "starboard")]` (or with
/// `, network = "direct-dial"` to pin the test to one network) on
/// `fn name(test: &mut Orchestrator)` expands to the ignored multi-thread
/// tokio test that builds the `Orchestrator` from the environment (or the
/// pinned network, whatever `E2E_NETWORK` says), replays the body to
/// configure it, runs it with screen recording, and asserts the report.
#[proc_macro_attribute]
pub fn e2e_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut game: Option<String> = None;
    let mut network: Option<String> = None;
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("game") {
            game = Some(meta.value()?.parse::<syn::LitStr>()?.value());
            Ok(())
        } else if meta.path.is_ident("network") {
            network = Some(meta.value()?.parse::<syn::LitStr>()?.value());
            Ok(())
        } else {
            Err(meta.error("e2e_test: expected game or network"))
        }
    });
    parse_macro_input!(attr with parser);
    let network_expr = match network.as_deref() {
        None => {
            quote! { freenet_libp2p_bevy_plugin::testing::Network::from_env() }
        }
        Some("fake") => {
            quote! { freenet_libp2p_bevy_plugin::testing::Network::Fake }
        }
        Some("mainnet") => {
            quote! { freenet_libp2p_bevy_plugin::testing::Network::Mainnet }
        }
        Some("local-gateways") => {
            quote! { freenet_libp2p_bevy_plugin::testing::Network::LocalGateways }
        }
        Some("direct-dial") => {
            quote! { freenet_libp2p_bevy_plugin::testing::Network::DirectDial }
        }
        Some(_) => {
            return quote! {
                compile_error!("e2e_test: network must be fake, mainnet, local-gateways or direct-dial");
            }
            .into();
        }
    };
    let Some(game) = game else {
        return quote! {
            compile_error!("#[e2e_test(game = \"...\")] needs a game name");
        }
        .into();
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
                #network_expr,
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
