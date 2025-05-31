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
    id: i32,
    name: String,
    time: ArcRwSignal<Vec<f32>>,
    position: ArcRwSignal<(i32, i32)>,
    rotation: ArcRwSignal<f32>,
}

#[component]
pub fn App(
    panel_size: ReadSignal<(i32, i32, i32, i32)>,
    config: RwSignal<Config>,
) -> impl IntoView {
    let (global_timer, set_global_timer) = signal(0.0);
    let (active_player, set_active_player) = signal(0);
    let (start_timer, set_start_timer) = signal(0.0);

    let now = Instant::now();
    let update = move || set_global_timer.set(now.elapsed().as_secs_f32());
    set_interval(update, Duration::from_millis(100));

    let players: RwSignal<Vec<Player>> = RwSignal::new(Vec::new());

    Effect::new(move || {
        players.update(|list| {
            list.resize_with(config.get().nplayers as usize, move || Player {
                id: 0,
                name: String::new(),
                time: ArcRwSignal::new(Vec::new()),
                position: ArcRwSignal::new((0, 0)),
                rotation: ArcRwSignal::new(0.0),
            });
            list.into_iter().enumerate().for_each(|(i, player)| {
                let pos = ((i as i32 + 1) * 120, 250);
                player.id = i as i32;
                player.name = config.get().names[i].clone();
                player.position.set(pos);
                player.time.set(Vec::new());
                player.rotation.set(0.0);

                logging::log!(
                    "Player {} name set to {}, position set to ({}, {})",
                    player.id,
                    player.name,
                    pos.0,
                    pos.1
                );
            });
        });
        logging::log!("{} players", config.get().nplayers);
    });
    let on_click = move || {
        let next: i32 = (active_player.get() + 1) % config.get().nplayers;
        logging::log!("calling increment on {}", next);
        set_active_player.set(next);
        set_start_timer.set(global_timer.get());
    };

    view! {
        <TimeTable players=players />

        <For each=move || players.get() key=|state| state.name.clone() let(child)>
            <Player player=child active_player on_click global_timer start_timer panel_size />
        </For>
    }
}

#[component]
fn Player(
    player: Player,
    active_player: ReadSignal<i32>,
    mut on_click: impl FnMut() + 'static,
    global_timer: ReadSignal<f32>,
    start_timer: ReadSignal<f32>,
    panel_size: ReadSignal<(i32, i32, i32, i32)>,
) -> impl IntoView {
    let current_panel_size = Rc::new(RefCell::new((0, 0, 0, 0)));
    let ppos = player.position.clone();
    {
        let current_panel_size = Rc::clone(&current_panel_size);
        Effect::new(move || {
            let new_size = panel_size.get();
            *current_panel_size.borrow_mut() = new_size;
            let (pos_x, pos_y) = ppos.get();
            let (x, y, panel_width, panel_height) = *current_panel_size.borrow();
            let new_x = (pos_x).max(x + 20).min(x + panel_width - 20);
            let new_y = (pos_y).max(y + 20).min(y + panel_height - 20);
            if pos_x != new_x || pos_y != new_y {
                ppos.set((new_x, new_y));
            }
        });
    }
    let player = player.clone();
    let handle_mouse_down = {
        let player = player.clone();
        let is_dragging = true;
        move |ev: web_sys::MouseEvent| {
            logging::log!("handle_mouse_down for player {}", player.name);

            let p_dragging = Rc::new(RefCell::new(is_dragging));
            let (px, py) = player.position.get();
            let offset_x = ev.client_x() - px;
            let offset_y = ev.client_y() - py;

            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            ev.prevent_default();

            let move_handler = {
                let player = player.clone();
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
                        player.position.set((new_x, new_y));
                    }
                }) as Box<dyn FnMut(_)>)
            };

            let player_name = player.name.clone();
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
    let ppos1 = player.position.clone();
    let ppos2 = player.position.clone();
    let p = player.clone();
    let p2 = player.clone();
    view! {
        <div
            class="absolute player-container"
            style:left=move || format!("{}px", ppos1.get().0)
            style:top=move || format!("{}px", ppos2.get().1)
            style:transform=move || format!("rotate({}deg)", p2.rotation.get())
        >
            <div class="name-tag-row">
                <button
                    class="rotation-button"
                    on:click=move |_| { p.rotation.set(p.rotation.get() + 45.0) }
                    title="Rotate player"
                ></button>
                <div
                    class="usertime-name-tag"
                    on:mousedown=handle_mouse_down
                    style="user-select: none;"
                    title="Drag to move"
                >
                    <p>{move || format!("{}", p.name)}</p>
                </div>
            </div>
            <UserTime player active_player on_click global_timer start_timer />
        </div>
    }
}

struct Clock {
    global_timer: ReadSignal<f32>,
    start_timer: ReadSignal<f32>,
    timer: RwSignal<f32>,
    active: RwSignal<bool>,
}

impl Clock {
    fn new(new_global_timer: ReadSignal<f32>, new_start_timer: ReadSignal<f32>) -> Self {
        let m = Self {
            global_timer: new_global_timer,
            start_timer: new_start_timer,
            timer: RwSignal::new(0.0),
            active: RwSignal::new(false),
        };
        Effect::new(move || {
            if m.active.get() {
                m.timer.set(m.global_timer.get() - m.start_timer.get());
            }
        });
        return m;
    }

    fn unpause(&self) {
        self.active.set(true)
    }

    fn pause(&self) {
        self.active.set(false)
    }
}

#[component]
fn UserTime(
    player: Player,
    active_player: ReadSignal<i32>,
    mut on_click: impl FnMut() + 'static,
    global_timer: ReadSignal<f32>,
    start_timer: ReadSignal<f32>,
) -> impl IntoView {
    let (utimer, set_utimer) = signal(0.0);
    let (active, set_active) = signal(false);

    let id = player.id;
    Effect::new(move || {
        set_active.set(id == active_player.get());
        logging::log!("active_player {}", active_player.get());
    });
    Effect::new(move || {
        if active.get() {
            set_utimer.set(global_timer.get() - start_timer.get());
        }
    });
    let ptime = player.time.clone();
    let mut toggle = move || {
        on_click();
        ptime.update(|timer| timer.push(utimer.get()));
        logging::log!(
            "pushing time on player {}: t{}",
            id,
            ptime.get().last().unwrap()
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
