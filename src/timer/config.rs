use leptos::{logging, prelude::*};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub nplayers: i32,
    pub names: Vec<String>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            nplayers: 0,
            names: vec![],
        }
        //Self {nplayers: 2, names: vec!["Player 1".to_string(), "Player 2".to_string()]}
    }
}

#[component]
pub fn Configuration(config_signal: RwSignal<Config>) -> impl IntoView {
    let config = RwSignal::new(Config::new());
    view! {
        <div class="config-section config-container">
            <div class="config-layout">
                <div class="config-left">
                    <div class="config-label">Number of Players</div>
                    <input
                        type="number"
                        class="config-input config-player-number"
                        on:input=move |ev| {
                            if let Ok(num) = event_target_value(&ev).parse() {
                                config.update(|c| c.nplayers = num);
                                config.update(|c| c.names.resize(num as usize, String::new()));
                                logging::log!(
                                    "Number of players changed to {}", config.get().nplayers
                                );
                            }
                        }
                        prop:value=move || config.get().nplayers.to_string()
                    />
                </div>
                <div class="config-right">
                    <div class="config-player-list">
                        <For
                            each=move || (0..config.get().nplayers)
                            key=move |i| i.clone()
                            let(child)
                        >
                            <input
                                type="text"
                                class="config-text-input config-player-item"
                                on:input=move |ev| {
                                    let name = event_target_value(&ev);
                                    config.update(|c| c.names[child as usize] = name);
                                    logging::log!(
                                        "Player name changed to {}", config.get().names[child as usize]
                                    );
                                }
                                prop:value=move || config.get().names[child as usize].clone()
                            />
                        </For>
                    </div>
                </div>
            </div>
            <button
                class="config-save-button"
                on:click=move |_| {
                    logging::log!("Configuration saved, {}", config.get().nplayers);
                    config_signal.set(config.get());
                }
            >
                "Save"
            </button>
        </div>
    }
}
