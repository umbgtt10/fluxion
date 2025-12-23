// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

/// Simple chart renderer for visualizing sensor data
pub struct Chart {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    data_points: Vec<f64>,
    max_points: usize,
}

impl Chart {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, wasm_bindgen::JsValue> {
        let ctx = canvas
            .get_context("2d")?
            .ok_or("Failed to get 2d context")?
            .dyn_into::<CanvasRenderingContext2d>()?;

        Ok(Self {
            canvas,
            ctx,
            data_points: Vec::new(),
            max_points: 100,
        })
    }

    pub fn add_point(&mut self, value: f64) {
        self.data_points.push(value);
        if self.data_points.len() > self.max_points {
            self.data_points.remove(0);
        }
    }

    pub fn render(&self) -> Result<(), wasm_bindgen::JsValue> {
        let width = self.canvas.width() as f64;
        let height = self.canvas.height() as f64;

        // Clear canvas
        self.ctx.clear_rect(0.0, 0.0, width, height);

        // Draw background
        self.ctx.set_fill_style(&"#1a1a2e".into());
        self.ctx.fill_rect(0.0, 0.0, width, height);

        if self.data_points.is_empty() {
            return Ok(());
        }

        // Draw grid lines
        self.ctx.set_stroke_style(&"#16213e".into());
        self.ctx.set_line_width(1.0);
        for i in 0..5 {
            let y = (i as f64 * height) / 4.0;
            self.ctx.begin_path();
            self.ctx.move_to(0.0, y);
            self.ctx.line_to(width, y);
            self.ctx.stroke();
        }

        // Draw data line
        self.ctx.set_stroke_style(&"#0f3460".into());
        self.ctx.set_line_width(2.0);
        self.ctx.begin_path();

        let x_step = width / (self.max_points as f64);
        let y_scale = height * 0.8;
        let y_offset = height * 0.1;

        for (i, &value) in self.data_points.iter().enumerate() {
            let x = i as f64 * x_step;
            let y = y_offset + (1.0 - value) * y_scale;

            if i == 0 {
                self.ctx.move_to(x, y);
            } else {
                self.ctx.line_to(x, y);
            }
        }

        self.ctx.stroke();

        Ok(())
    }
}
