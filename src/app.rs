use std::io::Cursor;
use std::io::Seek;

use brainfuck_rs::error::InterpreterError;
use brainfuck_rs::lexer::LexerToken;
use brainfuck_rs::lexer::LexerTokenMode;
use brainfuck_rs::parser::Parser;
use brainfuck_rs::parser::ParserBoundnessMode;
use brainfuck_rs::parser::ParserError;

const DEFAULT_MEMORY_ADDRESS: usize = 30000;
const DEFAULT_PROGRAM: &str = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++."; // This is the Brainfuck program for "Hello World!"

pub struct BrainfuckGui {
    running: bool,
    program: String,
    error_message: Option<String>,
    input: Cursor<String>,
    output: Vec<u8>,
    bf_parser: Parser,
}

impl BrainfuckGui {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let program = DEFAULT_PROGRAM.to_string();

        // Load the default program into the parser on initialization
        let mut parser = Parser::<DEFAULT_MEMORY_ADDRESS>::new(
            LexerTokenMode::Strict,
            ParserBoundnessMode::Strict,
        );
        let _ = parser.load_program(&program);

        Self {
            running: false,
            program,
            error_message: None,
            input: Cursor::new(String::new()),
            output: Vec::new(),
            bf_parser: parser,
        }
    }

    fn handle_error<E: std::fmt::Display>(&mut self, error: E) {
        let msg = error.to_string();
        tracing::error!("brainfuck_error = {msg}");

        self.error_message = Some(msg);
        self.running = false;
    }

    fn clear_error(&mut self) {
        self.error_message = None;
    }

    fn reset_fsm(&mut self) {
        self.clear_error();

        let _ = self.input.rewind();
        self.output.clear();

        self.bf_parser.reset();
    }

    fn instruction_at(&self, index: Option<usize>) -> Option<(usize, LexerToken)> {
        index.and_then(|index| {
            self.bf_parser
                .program_instruction(index)
                .copied()
                .map(|token| (index, token))
        })
    }

    fn create_label_program_counter(
        ui: &mut egui::Ui,
        instruction: Option<(usize, LexerToken)>,
        text_label: &str,
    ) {
        let text = egui::RichText::new(match instruction {
            Some((i, _)) => format!("{text_label} [{i}]"),
            None => text_label.to_owned(),
        });

        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
            |ui| {
                ui.monospace(text);
            },
        );
    }

    fn create_label_program_instruction(
        ui: &mut egui::Ui,
        instruction: Option<(usize, LexerToken)>,
        empty_text: &str,
        highlight: bool,
    ) {
        let mut text = egui::RichText::new(match instruction {
            Some((_, c)) => format!("{c}"),
            None => format!("<{empty_text}>"),
        });
        text = if highlight {
            text.background_color(egui::Color32::from_rgb(50, 80, 50))
                .color(egui::Color32::WHITE)
        } else {
            text
        };

        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
            |ui| {
                ui.monospace(text);
            },
        );
    }

    fn create_label_memory_address(ui: &mut egui::Ui, mem_addr: usize, highlight: bool) {
        let mut text = egui::RichText::new(format!("{mem_addr}"));
        text = if highlight { text.underline() } else { text };

        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
            |ui| {
                ui.monospace(text);
            },
        );
    }

    fn create_label_memory_value(ui: &mut egui::Ui, mem_value: u8, highlight: bool) {
        let mut text = egui::RichText::new(format!("{mem_value}"));
        text = if highlight {
            text.background_color(egui::Color32::from_rgb(120, 60, 60))
                .color(egui::Color32::WHITE)
        } else {
            text
        };

        ui.with_layout(
            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
            |ui| {
                ui.monospace(text);
            },
        );
    }

    fn show_program_counter(&self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.horizontal_centered(|ui| {
                ui.heading("Program counter");
            });

            ui.add_space(4.0);

            egui::Grid::new("program_counter_grid")
                .spacing([20.0, 0.0])
                .num_columns(3)
                .max_col_width(50.0)
                .striped(true)
                .show(ui, |ui| {
                    // Get the instructions
                    let pc = self.bf_parser.program_counter();
                    let prev = self.instruction_at(if pc == 0 { None } else { Some(pc - 1) });
                    let curr = self.instruction_at(Some(pc));
                    let next = self.instruction_at(Some(pc + 1));

                    // Display the program counters
                    Self::create_label_program_counter(ui, prev, "Prev");
                    Self::create_label_program_counter(ui, curr, "Current");
                    Self::create_label_program_counter(ui, next, "Next");
                    ui.end_row();

                    // Display the program instructions
                    Self::create_label_program_instruction(ui, prev, "none", false);
                    Self::create_label_program_instruction(ui, curr, "end", true);
                    Self::create_label_program_instruction(ui, next, "none", false);
                    ui.end_row();
                });
        });
    }

    fn show_memory_viewer(&self, ui: &mut egui::Ui) {
        let mem_addr = self.bf_parser.memory_address();

        ui.group(|ui| {
            const MEMORY_COLUMN: usize = 10;

            ui.horizontal_centered(|ui| {
                ui.heading("Memory viewer");
            });

            ui.add_space(4.0);

            // Calculate the start and end memory addresses
            let start = mem_addr.saturating_sub((MEMORY_COLUMN / 2).max(
                MEMORY_COLUMN.saturating_sub(DEFAULT_MEMORY_ADDRESS.saturating_sub(1) - mem_addr),
            ));
            let end = (mem_addr
                .saturating_add((MEMORY_COLUMN / 2).max(MEMORY_COLUMN.saturating_sub(mem_addr))))
            .min(DEFAULT_MEMORY_ADDRESS.saturating_sub(1));

            egui::Grid::new("memory_grid")
                .spacing([20.0, 0.0])
                .striped(true)
                .num_columns(MEMORY_COLUMN + 1)
                .max_col_width(50.0)
                .show(ui, |ui| {
                    // Display the memory addresses
                    for i in start..=end {
                        Self::create_label_memory_address(ui, i, i == mem_addr);
                    }
                    ui.end_row();

                    // Display the memory values
                    for i in start..=end {
                        let memory_value =
                            self.bf_parser.memory_value(i).copied().unwrap_or_default();
                        Self::create_label_memory_value(ui, memory_value, i == mem_addr);
                    }
                    ui.end_row();
                });
        });
    }

    fn brainfuck_step(&mut self) {
        // Handle 'ParserError::ProgramFinished' has a normal error, and keep the other errors
        let result = self.bf_parser.step(&mut self.input, &mut self.output);
        if result == Err(InterpreterError::Parser(ParserError::ProgramFinished)) {
            self.running = false;
        } else {
            // Discard the 'Ok(())' result
            let _ = result.map_err(|err| self.handle_error(err));
        }
    }
}

impl eframe::App for BrainfuckGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.running {
            self.brainfuck_step();
        }

        egui::TopBottomPanel::top("brainfuck_code_panel").show(ctx, |ui| {
            ui.group(|ui| {
                ui.heading("Brainfuck code");
                if ui
                    .add_enabled(
                        !self.running,
                        egui::TextEdit::multiline(&mut self.program)
                            .desired_rows(4)
                            .desired_width(f32::INFINITY),
                    )
                    .changed()
                {
                    // Discard the 'Ok(())' result
                    let _ = self
                        .bf_parser
                        .load_program(&self.program)
                        .map_err(|err| self.handle_error(err));
                }

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if !self.running {
                        if ui.button("Start").clicked() {
                            tracing::info!("Starting Brainfuck interpreter");

                            self.running = true;
                        }
                    } else if ui.button("Stop").clicked() {
                        tracing::info!("Stopping Brainfuck interpreter");

                        self.running = false;
                    }

                    if ui
                        .add_enabled(!self.running, egui::Button::new("Step"))
                        .clicked()
                    {
                        tracing::info!("Doing a single step");

                        self.brainfuck_step();
                    }

                    if ui
                        .add_enabled(!self.running, egui::Button::new("Reset"))
                        .clicked()
                    {
                        tracing::info!("Resetting Brainfuck interpreter");

                        self.reset_fsm();
                    }
                });
            });

            ui.add_space(8.0);

            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                self.show_program_counter(ui);

                ui.add_space(12.0);

                self.show_memory_viewer(ui);
            });
        });

        // Get half width of the screen for side panels
        let half = ctx.screen_rect().width() * 0.5;

        egui::SidePanel::left("input_panel")
            .resizable(false)
            .exact_width(half)
            .show(ctx, |ui| {
                ui.heading("Input");

                ui.add_space(4.0);

                ui.add_enabled(
                    !self.running,
                    egui::TextEdit::multiline(self.input.get_mut())
                        .desired_rows(4)
                        .desired_width(f32::INFINITY),
                );
            });

        egui::SidePanel::right("output_panel")
            .resizable(false)
            .exact_width(half)
            .show(ctx, |ui| {
                ui.heading("Output");

                ui.add_space(4.0);

                let result =
                    String::from_utf8(self.output.clone()).map_err(|err| self.handle_error(err));
                if let Ok(mut output) = result {
                    ui.add(
                        egui::TextEdit::multiline(&mut output)
                            .interactive(false)
                            .desired_rows(4)
                            .desired_width(f32::INFINITY),
                    );

                    ui.add_space(6.0);

                    if ui.button("Copy").clicked() {
                        ctx.copy_text(output);
                    }
                }
            });

        // If the interpreter is running, request a repaint to update the UI
        if self.running {
            ctx.request_repaint();
        }
    }
}
