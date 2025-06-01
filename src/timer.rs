mod config;

pub use config::{Config, Configuration};

use leptos::{logging, prelude::*};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use wasm_bindgen::JsCast;
use web_time::Instant;

#[derive(Clone)]
struct Player {
    id: usize,
    name: RwSignal<String>,
    time: RwSignal<Vec<f32>>,
    position: ArcRwSignal<(i32, i32)>,
    rotation: RwSignal<f32>,
}
impl Player {
    fn new(id: usize, name: String) -> Self {
        Self {
            id,
            name: RwSignal::new(name),
            time: RwSignal::new(Vec::new()),
            position: ArcRwSignal::new(((id as i32 + 1) * 120, 250)),
            rotation: RwSignal::new(0.0),
        }
    }
}

#[component]
pub fn App(
    panel_size: ReadSignal<(i32, i32, i32, i32)>,
    config: RwSignal<Config>,
) -> impl IntoView {
    let background_timer = RwSignal::new(0.0);
    let active_game = RwSignal::new(false);
    let elapsed_time = RwSignal::new(0.0);
    let unpause_time = RwSignal::new(0.0);
    let (global_timer, set_global_timer) = signal(0.0);
    let (active_player, set_active_player) = signal(0);
    let (start_timer, set_start_timer) = signal(0.0);

    let now = Instant::now();
    let update = move || background_timer.set(now.elapsed().as_secs_f32());
    set_interval(update, Duration::from_millis(100));

    Effect::new(move || {
        if active_game.get() {
            set_global_timer.set(elapsed_time.get() + background_timer.get() - unpause_time.get());
        }
    });

    let pause_unpause = move |_| {
        active_game.set(!active_game.get());
        if active_game.get() {
            unpause_time.set(background_timer.get());
        } else {
            elapsed_time.set(global_timer.get());
        }
    };

    let players = RwSignal::new(Vec::new());

    let reset_players = move || {
        let count = config.get().nplayers;
        // Create a completely new vector of players with fresh signals.
        let new_players = (0..count)
            .map(|i| Player::new(i, config.get().names[i].clone()))
            .collect();
        players.set(new_players);
        logging::log!(
            "{} players, game {}",
            config.get().nplayers,
            config.get().game_counter
        );
    };
    let reset_game = move || {
        reset_players();
    };
    Effect::new(move |_| {
        logging::log!("resetting game from effect");
        reset_game();
    });
    let go_back = move |_| {
        let next: usize = match active_player.get() == 0 {
            true => config.get().nplayers - 1,
            false => active_player.get() - 1,
        };
        let next_player = &mut players.get()[next];
        let mut next_player_time = next_player.time.get();
        if next_player_time.len() > 0 {
            next_player_time.pop();
            next_player.time.set(next_player_time);
        }
        set_active_player.set(next);
        set_start_timer.set(global_timer.get());
    };

    let player_toggle = move || {
        let next: usize = (active_player.get() + 1) % config.get().nplayers;
        logging::log!("calling increment on {}", next);
        set_active_player.set(next);
        set_start_timer.set(global_timer.get());
    };

    view! {
        <div>
            <div>
                <button on:click=pause_unpause>
                    "start/stop"
                </button>
                <button on:click=go_back>
                    "back"
                </button>
                <button on:click=move |_| {
                    reset_game();
                }>
                    "reset"
                </button>
            </div>
            <TimeTable players=players />
        </div>
        <For each=move || players.get()
            key=move |state| state.name.clone()
            let:player
        >
            <Player player active_player player_toggle global_timer start_timer panel_size />
        </For>
    }
}

#[component]
fn Player(
    player: Player,
    active_player: ReadSignal<usize>,
    mut player_toggle: impl FnMut() + 'static,
    global_timer: ReadSignal<f32>,
    start_timer: ReadSignal<f32>,
    panel_size: ReadSignal<(i32, i32, i32, i32)>,
) -> impl IntoView {
    let current_panel_size = Rc::new(RefCell::new((0, 0, 0, 0)));

    let player_a1 = player.clone();
    {
        let current_panel_size = Rc::clone(&current_panel_size);
        Effect::new(move || {
            let new_size = panel_size.get();
            *current_panel_size.borrow_mut() = new_size;
            let (pos_x, pos_y) = player_a1.position.get();
            let (x, y, panel_width, panel_height) = *current_panel_size.borrow();
            let new_x = (pos_x).max(x + 20).min(x + panel_width - 20);
            let new_y = (pos_y).max(y + 20).min(y + panel_height - 20);
            if pos_x != new_x || pos_y != new_y {
                player_a1.position.set((new_x, new_y));
            }
        });
    }

    let player1 = player.clone();
    let handle_mouse_down = {
        let is_dragging = true;
        let player2 = player1.clone();
        move |ev: web_sys::MouseEvent| {
            logging::log!("handle_mouse_down for player {}", player2.name.get());

            let p_dragging = Rc::new(RefCell::new(is_dragging));
            let (px, py) = player2.position.get();
            let offset_x = ev.client_x() - px;
            let offset_y = ev.client_y() - py;

            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            ev.prevent_default();

            let player3 = player2.clone();
            let move_handler = {
                let p_dragging = Rc::clone(&p_dragging);
                let current_panel_size = Rc::clone(&current_panel_size);

                wasm_bindgen::closure::Closure::wrap(Box::new(move |ev: web_sys::MouseEvent| {
                    if *p_dragging.borrow() {
                        let (x, y, panel_width, panel_height) = *current_panel_size.borrow();
                        let new_x = (ev.client_x() - offset_x)
                            .max(x + 20)
                            .min(x + panel_width - 20);
                        let new_y = (ev.client_y() - offset_y)
                            .max(y + 20)
                            .min(y + panel_height - 20);
                        player3.position.set((new_x, new_y));
                    }
                }) as Box<dyn FnMut(_)>)
            };

            let player_name = player2.name.get().clone();
            let up_handler =
                wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::MouseEvent| {
                    if *p_dragging.borrow() {
                        logging::log!("handle_mouse_up for player {}", player_name);
                        p_dragging.replace(false);
                    }
                }) as Box<dyn FnMut(_)>);

            document
                .add_event_listener_with_callback(
                    "mousemove",
                    move_handler.as_ref().unchecked_ref(),
                )
                .unwrap();
            document
                .add_event_listener_with_callback("mouseup", up_handler.as_ref().unchecked_ref())
                .unwrap();

            move_handler.forget();
            up_handler.forget();
        }
    };

    let pos = player.position.clone();
    let pos2 = player.position.clone();
    let rot = player.rotation.clone();

    view! {
        <div
            style:position="absolute"
            style:left=move || format!("{}px", pos.get().0)
            style:top=move || format!("{}px", pos2.get().1)
            style:transform= move || format!("rotate({}deg)", rot.get())
            class="player-container"
        >
            <div class="name-tag-row">
                <button
                    class="rotation-button"
                    on:click=move |_| { player.rotation.set(player.rotation.get() + 45.0) }
                    title="Rotate player"
                ></button>
                <div
                    class="usertime-name-tag"
                    on:mousedown=handle_mouse_down
                    style="user-select: none;"
                    title="Drag to move"
                >
                    <p>{move || format!("{}", player.name.get())}</p>
                </div>
            </div>
            <UserTime player active_player on_click=player_toggle global_timer start_timer />
        </div>
    }
}

#[component]
fn UserTime(
    player: Player,
    active_player: ReadSignal<usize>,
    mut on_click: impl FnMut() + 'static,
    global_timer: ReadSignal<f32>,
    start_timer: ReadSignal<f32>,
) -> impl IntoView {
    let (utimer, set_utimer) = signal(0.0);
    let (active, set_active) = signal(false);

    Effect::new(move || {
        set_active.set(player.id == active_player.get());
        logging::log!("active_player {}", active_player.get());
    });
    Effect::new(move || {
        if active.get() {
            set_utimer.set(global_timer.get() - start_timer.get());
        }
    });
    let mut toggle = move || {
        on_click();
        player.time.update(|timer| timer.push(utimer.get()));
        logging::log!(
            "pushing time on player {}: t{}",
            player.id,
            player.time.get().last().unwrap()
        );
    };

    view! {
        <button
            class=move || {
                if active.get() {
                    "usertime-button usertime-button-active"
                } else {
                    "usertime-button usertime-button-inactive"
                }
            }

            style:color="#1a1a1a"
            on:click=move |_| {
                if active.get() {
                    toggle();
                }
            }
        >
            <p>{move || format!("{:.1}", utimer.get())}</p>
        </button>
    }
}

#[component]
fn TimeTable(players: RwSignal<Vec<Player>>) -> impl IntoView {
    view! {
        <div class="time-table-container">
            <table class="time-table">
                <thead>
                    <tr>
                        <th>"Name"</th>
                        {move || {
                            let rounds = players
                                .get()
                                .into_iter()
                                .map(|player| { player.time.get().len() })
                                .max()
                                .unwrap_or(0);
                            (0..rounds)
                                .into_iter()
                                .map(|i| {
                                    view! { <th>{format!("Round {}", i + 1)}</th> }
                                })
                                .collect_view()
                        }}
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        players
                            .get()
                            .into_iter()
                            .map(|player| {
                                view! {
                                    <tr>
                                        <td>{player.name}</td>
                                        {player
                                            .time
                                            .get()
                                            .into_iter()
                                            .map(|t| {
                                                let s: String;
                                                if t > 60.0 {
                                                    s = format!("{}m:{}s", t as u64 / 60, t as u64 % 60);
                                                } else {
                                                    s = format!("{}s", t as u64);
                                                }
                                                view! { <td>{s}</td> }
                                            })
                                            .collect_view()}
                                    </tr>
                                }
                            })
                            .collect_view()
                    }}

                </tbody>
            </table>
        </div>
    }
}
