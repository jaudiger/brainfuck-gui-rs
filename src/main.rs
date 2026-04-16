/*
 *
 * Copyright (c) Jérémy Audiger.
 * All rights reserved.
 *
 */

mod app;

std::cfg_select! {
    target_arch = "wasm32" => {
        use eframe::wasm_bindgen::prelude::*;

        fn main() {
            tracing_wasm::set_as_global_default();

            wasm_bindgen_futures::spawn_local(async {
                let canvas = eframe::web_sys::window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .get_element_by_id("egui-canvas")
                    .unwrap()
                    .dyn_into::<eframe::web_sys::HtmlCanvasElement>()
                    .unwrap();

                eframe::WebRunner::new()
                    .start(
                        canvas,
                        eframe::WebOptions::default(),
                        Box::new(|cc| Ok(Box::new(app::BrainfuckGui::new(cc)))),
                    )
                    .await
                    .unwrap();
            });
        }
    }
    _ => {
        fn main() -> eframe::Result {
            let native_options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size([1400.0, 350.0])
                    .with_resizable(false),
                ..Default::default()
            };

            eframe::run_native(
                "Brainfuck GUI",
                native_options,
                Box::new(|cc| Ok(Box::new(app::BrainfuckGui::new(cc)))),
            )
        }
    }
}
