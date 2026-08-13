#[cfg(all(feature = "desktop", target_os = "macos"))]
mod native_menu;

#[cfg(feature = "desktop")]
#[derive(Debug)]
enum AppEvent {
    #[cfg(target_os = "macos")]
    Menu(muda::MenuEvent),
}

#[cfg(feature = "desktop")]
fn main() -> anyhow::Result<()> {
    use roc_datastar::DesktopServer;
    use tao::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoopBuilder, EventLoopWindowTarget},
        window::{Window, WindowBuilder},
    };
    use tracing_subscriber::EnvFilter;
    use wry::WebViewBuilder;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .compact()
        .init();

    let runtime = tokio::runtime::Runtime::new()?;
    let server = runtime.block_on(DesktopServer::start())?;
    tracing::info!(address = %server.address(), "desktop HTTP server listening");

    if std::env::args().any(|argument| argument == "--serve-only") {
        println!("{}", server.bootstrap_url());
        runtime.block_on(std::future::pending::<()>());
        unreachable!();
    }

    fn create_window(
        event_loop: &EventLoopWindowTarget<AppEvent>,
        url: &str,
    ) -> anyhow::Result<(Window, wry::WebView)> {
        let window = WindowBuilder::new()
            .with_title("Roc Datastar")
            .with_inner_size(tao::dpi::LogicalSize::new(1040.0, 760.0))
            .with_min_inner_size(tao::dpi::LogicalSize::new(720.0, 560.0))
            .build(event_loop)?;

        let webview = WebViewBuilder::new()
            .with_url(url)
            .with_devtools(cfg!(debug_assertions))
            .build(&window)?;

        Ok((window, webview))
    }

    let event_loop = EventLoopBuilder::<AppEvent>::with_user_event().build();

    #[cfg(target_os = "macos")]
    let _native_menu = native_menu::NativeMenu::install(event_loop.create_proxy())?;

    let bootstrap_url = server.bootstrap_url().to_owned();
    let mut desktop_window = Some(create_window(&event_loop, &bootstrap_url)?);

    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;
        // Keep the async runtime alive for as long as the native event loop.
        let _runtime = &runtime;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                #[cfg(target_os = "macos")]
                {
                    // Match normal macOS lifecycle behavior: closing the last
                    // window leaves the app running and available in the menu bar.
                    desktop_window = None;
                }
                #[cfg(not(target_os = "macos"))]
                {
                    server.shutdown();
                    *control_flow = ControlFlow::Exit;
                }
            }
            #[cfg(target_os = "macos")]
            Event::Reopen {
                has_visible_windows,
                ..
            } if !has_visible_windows => match create_window(event_loop, &bootstrap_url) {
                Ok(window) => desktop_window = Some(window),
                Err(error) => tracing::error!(%error, "failed to reopen window"),
            },
            #[cfg(target_os = "macos")]
            Event::UserEvent(AppEvent::Menu(menu_event)) => {
                if let Some((_, webview)) = &desktop_window {
                    if native_menu::is(&menu_event, native_menu::RELOAD_ID)
                        && let Err(error) = webview.reload()
                    {
                        tracing::error!(%error, "failed to reload webview");
                    }
                    #[cfg(debug_assertions)]
                    {
                        if native_menu::is(&menu_event, native_menu::WEB_INSPECTOR_ID) {
                            if webview.is_devtools_open() {
                                webview.close_devtools();
                            } else {
                                webview.open_devtools();
                            }
                        }
                    }
                }
            }
            Event::LoopDestroyed => server.shutdown(),
            _ => {}
        }
    });
}

#[cfg(not(feature = "desktop"))]
fn main() {
    eprintln!("The desktop binary requires the default `desktop` feature.");
}
