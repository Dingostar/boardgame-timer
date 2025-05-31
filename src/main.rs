mod timer;

use timer::{App, Config, Configuration};
use leptos::prelude::*;
use leptos::{logging, html, web_sys::*};
use wasm_bindgen::JsCast;


fn main() {
    console_error_panic_hook::set_once();

    leptos::mount::mount_to_body(Windows);
}


#[component]
fn Windows() -> impl IntoView {

    let bottom_panel_ref = NodeRef::<html::Div>::new();
    let (panel_size, set_panel_size) = signal((0, 0, 10000, 10000)); // Track size
    let config = RwSignal::new(Config::new());

    // Set up ResizeObserver
    Effect::new(move || {
        if let Some(panel) = bottom_panel_ref.get() {
            let element: &web_sys::HtmlDivElement = panel.as_ref();

            if let Some(panel) = bottom_panel_ref.get() {
                logging::log!("Element bounds: x{}, y{}", panel.get_bounding_client_rect().x(), panel.get_bounding_client_rect().y());

                let callback = wasm_bindgen::closure::Closure::wrap(Box::new(move |entries: js_sys::Array| {
                    if entries.get(0).dyn_into::<web_sys::ResizeObserverEntry>().is_ok() {
                        let panel_rect = panel.get_bounding_client_rect();
                        set_panel_size.set((panel_rect.x() as i32, panel_rect.y() as i32, panel_rect.width() as i32, panel_rect.height() as i32));
                        logging::log!("Panel resized to: {}x{}", panel_rect.width() as i32, panel_rect.height() as i32);
                    }
                }) as Box<dyn FnMut(js_sys::Array)>);

                let observer = web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref()).unwrap();
                observer.observe(element);

                callback.forget();
            }
        }
    });

    let (show, set_show) = signal(false);
    view! {
        <div class="resizable-horizontal-container">
            {move || {
                view! {
                    <div
                        class="top-panel"
                        style:display=move || if show.get() { "block" } else { "none" }
                    >
                        <Configuration config_signal=config />
                    </div>
                }
            }}
            <div
                style:height="8px"
                style:background-color="gray"
                style:display="flex"
                style:justify-content="center"
                style:align-items="center"
                style:position="relative"
            >
                <button
                    style:cursor="pointer"
                    style:width="20px"
                    style:height="20px"
                    style:position="absolute"
                    style:background-color="#666"
                    style:border="1px solid #999"
                    style:border-radius="3px"
                    style:display="flex"
                    style:align-items="center"
                    style:justify-content="center"
                    style:color="white"
                    style:font-size="25px"
                    on:click=move |_| {
                        set_show.set(!show.get());
                        if show.get() {
                            logging::log!("expand");
                        } else {
                            logging::log!("collapse");
                        }
                    }
                >
                    {move || if show.get() { "▴" } else { "▾" }}
                </button>
            </div>
            <div
                id=".timer-panel"
                node_ref=bottom_panel_ref
                class="bottom-panel"
                style:height="100%"
            >
                <App panel_size config />
            </div>
        </div>
    }
}
