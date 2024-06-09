use dioxus::html::input_data::MouseButton;
use image::io::Reader as ImageReader;
use std::io::Cursor;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use dioxus::prelude::*;
use wasm_bindgen::{prelude::*, Clamped};

use crate::inference::{inference, ImageClassifier, ModelType};
use crate::model::mnist::Model;

#[component]
pub(crate) fn App() -> Element {
    let device = Default::default();
    let classifier = ImageClassifier { model: ModelType::WithNdArrayBackend(Model::new(&device))};
    let pos = Point::new();
    let pos_down = pos.clone();
    let pos_enter = pos.clone();
    let pos_move = pos.clone();

    rsx! {
        link { rel: "stylesheet", href: "main.css" }
        link {
            rel: "stylesheet",
            href: "https://cdnjs.cloudflare.com/ajax/libs/font-awesome/4.7.0/css/font-awesome.min.css"
        }
        section { class: "container max-w-fit",
            div { id: "header",
                class: "flex flex-row sticky items-center justify-center z-10",
                // img { src: "assets/logo.png", class: "w-24 h-24" }
                h1 {
                    class: "text-4xl font-bold mb-4",
                    style:"font-family:'0xProto Regular",
                    "Detypstify"
                }
            div {
                class: "flex justify-center",
                a {
                    target: "_blank",
                    href: "https://github.com/DieracDelta/detypstify",
                    i { class: "fa fa-github", }
                }
            }
            }
            div { class: "flex align-middle justify-center mb-4",
                canvas {
                    id: "canvas",
                    class: "border border-gray-700 bg-gray-800",
                    width: "400",
                    height: "300",
                    onmousedown:
                    move |evt| {
                      set_position(evt, pos_down.clone());
                    },
                    onmouseenter : move |evt| {
                      set_position(evt, pos_enter.clone());
                    },
                    onmouseup: move |_| {
                        // TODO: capture the image and run prediction
                        set_output("out1", "1. 0");
                    },
                    onmousemove: move |event| {
                        let maybe_canvas = draw(event, pos_move.clone());
                        if let Some(cv) = maybe_canvas {
                            use std::ops::Deref;
                            let cv2 : &[u8] = cv.deref();
                            inference(&classifier, cv2);
                        }

                    },
                }
            }
            div { class: "flex align-middle justify-center mb-4",
                div {
                    // make the class be dark and matching with the theme
                    class: "button-like",
                    tabindex: "0",
                    id: "btn",
                    onclick: move |_| {
                        let canvas = web_sys::window().unwrap().document().unwrap()
                            .get_element_by_id("canvas").unwrap();
                        let context = canvas.dyn_into::<web_sys::HtmlCanvasElement>()
                            .unwrap().get_context("2d").unwrap().unwrap()
                            .dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();
                        context.clear_rect(0.0, 0.0, 400.0, 300.0);
                        clear_outputs();
                    },
                    style:"font-family:'0xProto Regular",
                    "Clear"
                }
                div { class: "flex justify-center mb-4 button-like",
                  style:"font-family:'0xProto Regular",
                  "Upload Image",
                    input {
                        r#type: "hidden",
                        id: "imageUpload",
                        accept: "image/*",
                        onchange: move |evt| {
                            async move {
                                    if let Some(file_engine) = evt.files() {
                                    let files = file_engine.files();
                                    for file_name in &files {
                                        if let Some(bytes) = file_engine.read_file(file_name).await
                                        {
                                            let image = ImageReader::new(Cursor::new(bytes))
                                                .with_guessed_format().unwrap()
                                                .decode().unwrap();
                                            tracing::debug!("Image width: {}, height: {}",
                                                image.width(), image.height());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            OutputModel { name: "out1", res: "1." }
            OutputModel { name: "out2", res: "2." }
            OutputModel { name: "out3", res: "3." }
        }
    }
}

fn draw(event: MouseEvent, pos: Point) -> Option<Clamped<Vec<u8>>> {
    if event.held_buttons().contains(MouseButton::Primary) {
        let coords = event.element_coordinates();
        let canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("canvas")
            .unwrap();
        let context = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap()
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();
        context.begin_path();
        let (x, y) = pos.get_coords();
        context.move_to(x, y);
        set_position(event, pos);

        context.line_to(coords.x, coords.y);
        context.set_stroke_style(&JsValue::from_str("white"));
        context.set_line_width(5.0);
        context.stroke();

        Some(context.get_image_data(0.0, 0.0, 400.0, 300.0).unwrap().data())
    } else {
        None
    }
}

fn set_output(name: &str, res: &str) {
    let out = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id(name)
        .unwrap();
    out.set_text_content(Some(res));
}

fn clear_outputs() {
    set_output("out1", "");
    set_output("out2", "");
    set_output("out3", "");
}

#[component]
fn OutputModel(name: String, res: String) -> Element {
    rsx! {
        div {
            class: "flex justify-left",
            h1 {
                class: "text-2xl font-bold ",
                "{res}"
            }
            p {id : "{name}",
            class: "ml-8 mt-1",
            ""
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Point {
    x: Arc<AtomicU64>,
    y: Arc<AtomicU64>,
}

impl Point {
    fn new() -> Self {
        Self {
            x: Arc::default(),
            y: Arc::default(),
        }
    }

    fn get_coords(&self) -> (f64, f64) {
        (
            self.x.load(Ordering::SeqCst) as f64,
            self.y.load(Ordering::SeqCst) as f64,
        )
    }
}

fn set_position(event: MouseEvent, pos: Point) {
    let coords = event.element_coordinates();
    pos.x.store(coords.x as u64, Ordering::SeqCst);
    pos.y.store(coords.y as u64, Ordering::SeqCst);
}
