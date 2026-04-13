use eframe::egui;
use image;
use num_bigint::BigInt;
use num_rational::BigRational;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use tempfile::NamedTempFile;

#[derive(Default)]
struct App {
    input: String,
    result_text: String,
    status: String,
    preview_texture: Option<egui::TextureHandle>,
    steps_preview_texture: Option<egui::TextureHandle>,
    processing: bool,
    show_steps: bool,
    steps: Vec<String>,
    warning: Option<String>,
    final_png_bytes: Option<Vec<u8>>,
    steps_png_bytes: Option<Vec<u8>>,
    rx: Option<mpsc::Receiver<Result<(String, String, String, Vec<u8>, Option<Vec<String>>, Option<Vec<u8>>, Option<String>), String>>>,
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1100.0, 850.0]),
        ..Default::default()
    };
    eframe::run_native(
        "rusty_fractions",
        options,
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.processing {
            ctx.request_repaint_after(std::time::Duration::from_millis(30));
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("rusty_fractions");
            ui.add_space(20.0);

            ui.label("Expression:");
            ui.add_sized(
                [ui.available_width(), 90.0],
                egui::TextEdit::singleline(&mut self.input)
                    .font(egui::FontId::new(48.0, egui::FontFamily::Proportional)),
            );
            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui
                    .add_sized(
                        [260.0, 80.0],
                        egui::Button::new("Convert")
                            .fill(egui::Color32::from_rgb(0, 180, 0)),
                    )
                    .clicked()
                    && !self.processing
                {
                    self.start_conversion(ctx);
                }

                if ui
                    .add_sized(
                        [260.0, 80.0],
                        egui::Button::new("Clear").fill(egui::Color32::from_rgb(200, 0, 0)),
                    )
                    .clicked()
                {
                    self.clear();
                }

                if self.preview_texture.is_some() {
                    if ui
                        .add_sized(
                            [260.0, 80.0],
                            egui::Button::new("Save Renders")
                                .fill(egui::Color32::from_rgb(0, 100, 200)),
                        )
                        .clicked()
                    {
                        self.save_rendered_images();
                    }
                }
            });

            ui.checkbox(&mut self.show_steps, "Show step-by-step calculation");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Result:");
                if self.processing {
                    ui.add(egui::widgets::Spinner::new());
                }
            });

            if !self.result_text.is_empty() {
                ui.monospace(&self.result_text);
            }
            if !self.status.is_empty() {
                if self.status.starts_with("Error:") {
                    ui.colored_label(
                        egui::Color32::YELLOW,
                        egui::RichText::new(&self.status).strong(),
                    );
                } else {
                    ui.label(&self.status);
                }
            }
            if let Some(w) = &self.warning {
                ui.colored_label(egui::Color32::YELLOW, egui::RichText::new(w).strong());
            }

            if !self.steps.is_empty() {
                ui.separator();
                ui.label("Step-by-step solution (text):");
                egui::ScrollArea::vertical()
                    .max_height(280.0)
                    .id_salt("steps_text_scroll")
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        for step in &self.steps {
                            ui.label(egui::RichText::new(step).monospace());
                        }
                    });
            }

            if self.steps_preview_texture.is_some() || (self.show_steps && !self.steps.is_empty()) {
                ui.separator();
                ui.label("Rendered Step-by-Step:");
                egui::ScrollArea::vertical()
                    .max_height(520.0)
                    .id_salt("rendered_steps_scroll")
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        if let Some(texture) = &self.steps_preview_texture {
                            ui.centered_and_justified(|ui| {
                                ui.image((texture.id(), texture.size_vec2()));
                            });
                        } else if self.show_steps {
                            ui.centered_and_justified(|ui| {
                                ui.label("(Rendered steps will appear here)");
                            });
                        }
                    });
            }

            ui.separator();
            ui.label("Final Rendered Math Preview:");
            egui::ScrollArea::both().show(ui, |ui| {
                if let Some(texture) = &self.preview_texture {
                    ui.centered_and_justified(|ui| {
                        ui.image((texture.id(), texture.size_vec2()));
                    });
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("(Preview appears here after conversion)");
                    });
                }
            });
        });

        if let Some(rx) = &self.rx {
            if let Ok(result) = rx.try_recv() {
                self.processing = false;
                self.rx = None;
                match result {
                    Ok((md, tex, answer, png_bytes, opt_steps, opt_steps_png, warning_opt)) => {
                        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                        let _ = fs::write(cwd.join("math_output.md"), md);
                        let _ = fs::write(cwd.join("math_output.tex"), tex);

                        self.result_text = format!("Exact answer: {}", answer);
                        self.status = "Done".to_string();
                        self.steps = opt_steps.unwrap_or_default();
                        self.warning = warning_opt;
                        self.final_png_bytes = Some(png_bytes.clone());
                        self.steps_png_bytes = opt_steps_png.clone();

                        if let Ok(image) = image::load_from_memory(&png_bytes) {
                            let size = [image.width() as usize, image.height() as usize];
                            let rgba = image.to_rgba8();
                            let pixels = rgba.as_flat_samples().as_slice().to_vec();
                            self.preview_texture = Some(ctx.load_texture(
                                "math_preview",
                                egui::ColorImage::from_rgba_unmultiplied(size, &pixels),
                                Default::default(),
                            ));
                        }
                        if let Some(steps_png) = opt_steps_png {
                            if let Ok(image) = image::load_from_memory(&steps_png) {
                                let size = [image.width() as usize, image.height() as usize];
                                let rgba = image.to_rgba8();
                                let pixels = rgba.as_flat_samples().as_slice().to_vec();
                                self.steps_preview_texture = Some(ctx.load_texture(
                                    "steps_preview",
                                    egui::ColorImage::from_rgba_unmultiplied(size, &pixels),
                                    Default::default(),
                                ));
                            }
                        }
                    }
                    Err(e) => self.status = format!("Error: {}", e),
                }
            }
        }
    }
}

impl App {
    fn start_conversion(&mut self, ctx: &egui::Context) {
        let input = self.input.trim().to_string();
        if input.is_empty() {
            self.status = "Enter an expression".to_string();
            return;
        }
        self.processing = true;
        self.status.clear();
        self.steps.clear();
        self.steps_preview_texture = None;
        self.warning = None;
        self.final_png_bytes = None;
        self.steps_png_bytes = None;

        let show_steps = self.show_steps;
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);

        thread::spawn(move || {
            let result = process_expression(&input, show_steps);
            let _ = tx.send(result);
        });

        ctx.request_repaint();
    }

    fn clear(&mut self) {
        self.input.clear();
        self.result_text.clear();
        self.status.clear();
        self.steps.clear();
        self.preview_texture = None;
        self.steps_preview_texture = None;
        self.warning = None;
        self.final_png_bytes = None;
        self.steps_png_bytes = None;
        self.processing = false;
        self.rx = None;
    }

    fn save_rendered_images(&mut self) {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut saved = vec![];
        if let Some(bytes) = &self.final_png_bytes {
            let path = cwd.join("final_render.png");
            if fs::write(&path, bytes).is_ok() {
                saved.push("final_render.png");
            }
        }
        if let Some(bytes) = &self.steps_png_bytes {
            let path = cwd.join("steps_render.png");
            if fs::write(&path, bytes).is_ok() {
                saved.push("steps_render.png");
            }
        }
        if !saved.is_empty() {
            self.status = format!("Saved: {}", saved.join(" + "));
        } else {
            self.status = "Nothing to save yet".to_string();
        }
    }
}

// ====================== PARSER + EVALUATOR ======================

#[derive(Debug)]
enum Expr {
    Num(BigRational),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
    Paren(Box<Expr>),
}

fn process_expression(
    input: &str,
    show_steps: bool,
) -> Result<(String, String, String, Vec<u8>, Option<Vec<String>>, Option<Vec<u8>>, Option<String>), String>
{
    let cleaned = input
        .replace(['[', '{'], "(")
        .replace([']', '}'], ")")
        .replace(" ", "");
    let tokens = tokenize(&cleaned)?;
    let (expr, _) = parse_expr(&tokens, 0)?;

    let mut step_list = vec![];
    let result = if show_steps {
        evaluate_with_steps(&expr, &mut step_list)?
    } else {
        evaluate(&expr)?
    };

    let filtered_steps: Vec<String> = step_list
        .into_iter()
        .filter(|step| {
            let s = step.trim();
            !(s.contains(" ÷ ") && s.split(" = ").next().map_or(false, |left| {
                left.chars().filter(|c| c.is_numeric()).count() <= 2
            }))
        })
        .collect();

    let steps_opt = if show_steps && !filtered_steps.is_empty() {
        Some(filtered_steps)
    } else {
        None
    };

    let slash_count = cleaned.chars().filter(|&c| c == '/').count();
    let warning = if slash_count >= 3 {
        Some(
            "Warning: This expression contains 3 or more divisions.\n\
            Divisions are left-associative: a/b/c/d = ((a/b)/c)/d.\n\
            Consider adding parentheses for clarity.".to_string()
        )
    } else {
        None
    };

    let expr_latex = to_latex(&expr);
    let result_latex = to_latex_result(&result);

    let markdown = format!(
        r#"# Math Expression Result
**Expression:**
\[
{}
\]
**Result:**
\[
{} = {}
\]
Exact value: `{}`
"#,
        expr_latex, expr_latex, result_latex, result
    );

    let latex_doc = format!(
        r#"\documentclass[11pt]{{article}}
\usepackage{{amsmath}}
\usepackage[margin=0.3in]{{geometry}}
\begin{{document}}
\centering
\[
{}
\]
\[
{} = {}
\]
\end{{document}}
"#,
        expr_latex, expr_latex, result_latex
    );

    let png_bytes = render_to_png(&expr, &result)?;
    let steps_png_opt = if show_steps && steps_opt.is_some() {
        Some(render_steps_to_png(steps_opt.as_ref().unwrap())?)
    } else {
        None
    };

    Ok((
        markdown,
        latex_doc,
        result.to_string(),
        png_bytes,
        steps_opt,
        steps_png_opt,
        warning,
    ))
}

fn tokenize(s: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_numeric() || c == '.' {
            let mut num = String::new();
            let mut dot_seen = false;
            while i < chars.len() {
                let ch = chars[i];
                if ch.is_numeric() {
                    num.push(ch);
                } else if ch == '.' {
                    if dot_seen {
                        return Err("Multiple decimal points in number".to_string());
                    }
                    dot_seen = true;
                    num.push(ch);
                } else {
                    break;
                }
                i += 1;
            }
            tokens.push(num);
            continue;
        }
        if "()+*-/".contains(c) {
            tokens.push(c.to_string());
            i += 1;
            continue;
        }
        return Err(format!("Invalid character: '{}'", c));
    }
    Ok(tokens)
}

fn parse_number(token: &str) -> Result<BigRational, String> {
    if let Ok(n) = token.parse::<BigInt>() {
        Ok(BigRational::from_integer(n))
    } else if token.contains('.') {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 2 {
            return Err("Invalid decimal format".to_string());
        }
        let int_str = if parts[0].is_empty() { "0" } else { parts[0] };
        let frac_str = parts[1];
        if frac_str.is_empty() {
            return int_str
                .parse::<BigInt>()
                .map(BigRational::from_integer)
                .map_err(|e| e.to_string());
        }
        let int_part: BigInt = int_str.parse::<BigInt>().map_err(|e| e.to_string())?;
        let frac_len = frac_str.len();
        let denom = BigInt::from(10u32).pow(frac_len as u32);
        let frac_part: BigInt = frac_str.parse::<BigInt>().map_err(|e| e.to_string())?;
        let numer = int_part * &denom + frac_part;
        Ok(BigRational::new(numer, denom))
    } else {
        Err(format!("Invalid number: {}", token))
    }
}

fn parse_expr(tokens: &[String], mut pos: usize) -> Result<(Expr, usize), String> {
    let (mut left, new_pos) = parse_term(tokens, pos)?;
    pos = new_pos;
    while pos < tokens.len() {
        let op = &tokens[pos];
        if op != "+" && op != "-" { break; }
        pos += 1;
        let (right, new_pos) = parse_term(tokens, pos)?;
        left = if op == "+" {
            Expr::Add(Box::new(left), Box::new(right))
        } else {
            Expr::Sub(Box::new(left), Box::new(right))
        };
        pos = new_pos;
    }
    Ok((left, pos))
}

fn parse_term(tokens: &[String], mut pos: usize) -> Result<(Expr, usize), String> {
    let (mut left, new_pos) = parse_factor(tokens, pos)?;
    pos = new_pos;
    while pos < tokens.len() {
        let op_token = &tokens[pos];
        if op_token == "*" || op_token == "/" {
            pos += 1;
            let (right, new_pos) = parse_factor(tokens, pos)?;
            left = if op_token == "*" {
                Expr::Mul(Box::new(left), Box::new(right))
            } else {
                Expr::Div(Box::new(left), Box::new(right))
            };
            pos = new_pos;
        } else if starts_factor(op_token) {
            let (right, new_pos) = parse_factor(tokens, pos)?;
            left = Expr::Mul(Box::new(left), Box::new(right));
            pos = new_pos;
        } else {
            break;
        }
    }
    Ok((left, pos))
}

fn starts_factor(token: &str) -> bool {
    token == "(" || parse_number(token).is_ok()
}

fn parse_factor(tokens: &[String], mut pos: usize) -> Result<(Expr, usize), String> {
    if pos >= tokens.len() {
        return Err("Unexpected end of input".to_string());
    }
    let mut negative = false;
    while pos < tokens.len() && tokens[pos] == "-" {
        negative = !negative;
        pos += 1;
    }
    if pos >= tokens.len() {
        return Err("Unexpected end after unary operator".to_string());
    }
    let token = &tokens[pos];
    if let Ok(rat) = parse_number(token) {
        let expr = Expr::Num(rat);
        let final_expr = if negative { Expr::Neg(Box::new(expr)) } else { expr };
        return Ok((final_expr, pos + 1));
    }
    if token == "(" {
        let (inner, new_pos) = parse_expr(tokens, pos + 1)?;
        if new_pos >= tokens.len() || tokens[new_pos] != ")" {
            return Err("Missing closing parenthesis ')'".to_string());
        }
        let expr = Expr::Paren(Box::new(inner));
        let final_expr = if negative { Expr::Neg(Box::new(expr)) } else { expr };
        return Ok((final_expr, new_pos + 1));
    }
    Err(format!("Invalid token: {}", token))
}

fn evaluate(expr: &Expr) -> Result<BigRational, String> {
    match expr {
        Expr::Num(n) => Ok(n.clone()),
        Expr::Add(a, b) => Ok(evaluate(a)? + evaluate(b)?),
        Expr::Sub(a, b) => Ok(evaluate(a)? - evaluate(b)?),
        Expr::Mul(a, b) => Ok(evaluate(a)? * evaluate(b)?),
        Expr::Div(a, b) => {
            let b_val = evaluate(b)?;
            if b_val == BigRational::from_integer(BigInt::from(0)) {
                Err("Division by zero".to_string())
            } else {
                Ok(evaluate(a)? / b_val)
            }
        }
        Expr::Neg(a) => Ok(-evaluate(a)?),
        Expr::Paren(inner) => evaluate(inner),
    }
}

fn evaluate_with_steps(expr: &Expr, steps: &mut Vec<String>) -> Result<BigRational, String> {
    let val = match expr {
        Expr::Num(n) => n.clone(),
        Expr::Paren(inner) => evaluate_with_steps(inner, steps)?,
        Expr::Add(a, b) => {
            let va = evaluate_with_steps(a, steps)?;
            let vb = evaluate_with_steps(b, steps)?;
            let res = va + vb;
            steps.push(format!("{} + {} = {}", to_typst(a), to_typst(b), to_typst_result(&res)));
            res
        }
        Expr::Sub(a, b) => {
            let va = evaluate_with_steps(a, steps)?;
            let vb = evaluate_with_steps(b, steps)?;
            let res = va - vb;
            steps.push(format!("{} - {} = {}", to_typst(a), to_typst(b), to_typst_result(&res)));
            res
        }
        Expr::Mul(a, b) => {
            let va = evaluate_with_steps(a, steps)?;
            let vb = evaluate_with_steps(b, steps)?;
            let res = va * vb;
            steps.push(format!("{} × {} = {}", to_typst(a), to_typst(b), to_typst_result(&res)));
            res
        }
        Expr::Div(a, b) => {
            let va = evaluate_with_steps(a, steps)?;
            let vb = evaluate_with_steps(b, steps)?;
            if vb == BigRational::from_integer(BigInt::from(0)) {
                return Err("Division by zero".to_string());
            }
            let res = va / vb;
            steps.push(format!("{} ÷ {} = {}", to_typst(a), to_typst(b), to_typst_result(&res)));
            res
        }
        Expr::Neg(inner) => {
            let v = evaluate_with_steps(inner, steps)?;
            let res = -v;
            steps.push(format!("-{} = {}", to_typst(inner), to_typst_result(&res)));
            res
        }
    };
    Ok(val)
}

fn to_latex(expr: &Expr) -> String {
    match expr {
        Expr::Num(r) => {
            let (n, d) = (r.numer(), r.denom());
            if d == &BigInt::from(1) { n.to_string() } else { format!(r"\frac{{{}}}{{{}}}", n, d) }
        }
        Expr::Add(a, b) => format!("{} + {}", to_latex(a), to_latex(b)),
        Expr::Sub(a, b) => format!("{} - {}", to_latex(a), to_latex(b)),
        Expr::Mul(a, b) => format!("{} \\times {}", to_latex(a), to_latex(b)),
        Expr::Div(a, b) => format!(r"\frac{{{}}}{{{}}}", to_latex(a), to_latex(b)),
        Expr::Neg(inner) => format!("-{}", to_latex(inner)),
        Expr::Paren(inner) => format!("({})", to_latex(inner)),
    }
}

fn to_latex_result(r: &BigRational) -> String {
    let (n, d) = (r.numer(), r.denom());
    if d == &BigInt::from(1) { n.to_string() } else { format!(r"\frac{{{}}}{{{}}}", n, d) }
}

fn to_typst(expr: &Expr) -> String {
    match expr {
        Expr::Num(r) => {
            let (n, d) = (r.numer(), r.denom());
            if d == &BigInt::from(1) { n.to_string() } else { format!("{}/{}", n, d) }
        }
        Expr::Add(a, b) => format!("{} + {}", to_typst(a), to_typst(b)),
        Expr::Sub(a, b) => format!("{} - {}", to_typst(a), to_typst(b)),
        Expr::Mul(a, b) => format!("{} * {}", to_typst(a), to_typst(b)),
        Expr::Div(a, b) => format!("{}/{}", to_typst(a), to_typst(b)),
        Expr::Neg(inner) => format!("-{}", to_typst(inner)),
        Expr::Paren(inner) => format!("({})", to_typst(inner)),
    }
}

fn to_typst_result(r: &BigRational) -> String {
    let (n, d) = (r.numer(), r.denom());
    if d == &BigInt::from(1) { n.to_string() } else { format!("{}/{}", n, d) }
}

// ====================== TYPS CLI RENDERING ======================

fn render_to_png(expr: &Expr, result: &BigRational) -> Result<Vec<u8>, String> {
    let expr_str = to_typst(expr);
    let result_str = to_typst_result(result);
    let typst_content = format!(
        r#"#set page(width: auto, height: auto, margin: 0.8cm)
#set text(size: 32pt)
#align(center)[
  ${} = {}$
]
"#,
        expr_str, result_str
    );

    let typ_file = NamedTempFile::with_suffix(".typ").map_err(|e| e.to_string())?;
    let png_file = NamedTempFile::with_suffix(".png").map_err(|e| e.to_string())?;
    fs::write(typ_file.path(), typst_content).map_err(|e| e.to_string())?;

    let output = Command::new("typst")
        .args([
            "compile",
            "--format",
            "png",
            typ_file.path().to_str().unwrap(),
            png_file.path().to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Failed to run typst: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Typst error: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    fs::read(png_file.path()).map_err(|e| e.to_string())
}

fn render_steps_to_png(display_steps: &[String]) -> Result<Vec<u8>, String> {
    let typst_content = format!(
        r#"#set page(width: auto, height: auto, margin: 1cm)
#set text(size: 28pt, weight: "medium")
#align(left)[
#stack(
  spacing: 1.2em,
  {}
)
]
"#,
        display_steps
            .iter()
            .map(|s| format!("${}$", s.replace("=", "=&")))
            .collect::<Vec<_>>()
            .join(",\n ")
    );

    let typ_file = NamedTempFile::with_suffix(".typ").map_err(|e| e.to_string())?;
    let png_file = NamedTempFile::with_suffix(".png").map_err(|e| e.to_string())?;
    fs::write(typ_file.path(), typst_content).map_err(|e| e.to_string())?;

    let output = Command::new("typst")
        .args([
            "compile",
            "--format",
            "png",
            typ_file.path().to_str().unwrap(),
            png_file.path().to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Failed to run typst: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Typst error: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    fs::read(png_file.path()).map_err(|e| e.to_string())
}

// ====================== UNIT TESTS ======================
#[cfg(test)]
mod tests {
    use super::*;

    fn evaluate_expression(input: &str) -> Result<BigRational, String> {
        let cleaned = input
            .replace(['[', '{'], "(")
            .replace([']', '}'], ")")
            .replace(" ", "");
        let tokens = tokenize(&cleaned)?;
        let (expr, _) = parse_expr(&tokens, 0)?;
        evaluate(&expr)
    }

    #[test] fn test_implied_mul_2_1_3() { let res = evaluate_expression("2(1/3)").unwrap(); assert_eq!(res, BigRational::new(BigInt::from(2), BigInt::from(3))); }
    #[test] fn test_implied_mul_3_4_5() { let res = evaluate_expression("3(4+5)").unwrap(); assert_eq!(res, BigRational::from_integer(BigInt::from(27))); }
    #[test] fn test_implied_mul_1_2_4() { let res = evaluate_expression("(1+2)4").unwrap(); assert_eq!(res, BigRational::from_integer(BigInt::from(12))); }
    #[test] fn test_implied_mul_2_3_4() { let res = evaluate_expression("2(3)(4)").unwrap(); assert_eq!(res, BigRational::from_integer(BigInt::from(24))); }
    #[test] fn test_implied_mul_5_2_3_1_4() { let res = evaluate_expression("5(2/3 + 1/4)").unwrap(); assert_eq!(res, BigRational::new(BigInt::from(55), BigInt::from(12))); }
    #[test] fn test_chained_1_12_5_8() { let res = evaluate_expression("1/12/5/8").unwrap(); assert_eq!(res, BigRational::new(BigInt::from(1), BigInt::from(480))); }
    #[test] fn test_chained_1_2_3_4() { let res = evaluate_expression("1/2/3/4").unwrap(); assert_eq!(res, BigRational::new(BigInt::from(1), BigInt::from(24))); }
    #[test] fn test_chained_2_3_4_5() { let res = evaluate_expression("2/3/4/5").unwrap(); assert_eq!(res, BigRational::new(BigInt::from(1), BigInt::from(30))); }
    #[test] fn test_chained_1_12_5_5_8() { let res = evaluate_expression("1/12/5/5/8").unwrap(); assert_eq!(res, BigRational::new(BigInt::from(1), BigInt::from(2400))); }
    #[test] fn test_mixed_1_2_3_4_5_6() { let res = evaluate_expression("1/2 + 3(4-5)/6").unwrap(); assert_eq!(res, BigRational::from_integer(BigInt::from(0))); }
    #[test] fn test_mixed_2_1_3_4_5_3_7() { let res = evaluate_expression("2(1/3 + 4/5) - 3/7").unwrap(); assert_eq!(res, BigRational::new(BigInt::from(193), BigInt::from(105))); }
    #[test] fn test_mixed_neg_2_neg3_4_neg5() { let res = evaluate_expression("-2(-3+4)/(-5)").unwrap(); assert_eq!(res, BigRational::new(BigInt::from(2), BigInt::from(5))); }
    #[test] fn test_mixed_1_neg_neg2_3_4() { let res = evaluate_expression("1 - -2(3/4)").unwrap(); assert_eq!(res, BigRational::new(BigInt::from(5), BigInt::from(2))); }
    #[test] fn test_mixed_1_2_neg3_2_1_4() { let res = evaluate_expression("1/2 * -3(2+1/4)").unwrap(); assert_eq!(res, BigRational::new(BigInt::from(-27), BigInt::from(8))); }
    #[test] fn test_deep_2_3_4_5_6() { let res = evaluate_expression("2(3(4(5+6)))").unwrap(); assert_eq!(res, BigRational::from_integer(BigInt::from(264))); }
    #[test] fn test_deep_1_2_3_4_5_6_7_8() { let res = evaluate_expression("(1/2 + 3/4)(5/6 - 7/8)").unwrap(); assert_eq!(res, BigRational::new(BigInt::from(-5), BigInt::from(96))); }
    #[test] fn test_deep_1_2_3_4_5_6_7() { let res = evaluate_expression("1 + 2(3 + 4(5 - 6/7))").unwrap(); assert_eq!(res, BigRational::new(BigInt::from(281), BigInt::from(7))); }
    #[test] fn test_deep_neg_2_3_4_5_1_2() { let res = evaluate_expression("- (2/3)(4/5 - 1/2)").unwrap(); assert_eq!(res, BigRational::new(BigInt::from(-1), BigInt::from(5))); }
    #[test] fn test_big_123_456_789_1011() { let res = evaluate_expression("123/456 / 789/1011").unwrap(); assert_eq!(res, BigRational::new(BigInt::from(41), BigInt::from(121247208))); }
    #[test] fn test_big_987654_123456_1_2() { let res = evaluate_expression("987654/123456 + 1/2").unwrap(); assert_eq!(res, BigRational::new(BigInt::from(987654) * BigInt::from(2) + BigInt::from(123456), BigInt::from(123456) * BigInt::from(2))); }
    #[test] fn test_big_chain_mul() { let res = evaluate_expression("1/2 * 3/4 * 5/6 * 7/8").unwrap(); assert_eq!(res, BigRational::new(BigInt::from(105), BigInt::from(384))); }
    #[test] fn test_original_example() { let res = evaluate_expression("1/2-(1/4-4/5)*2/9").unwrap(); assert_eq!(res, BigRational::new(BigInt::from(28), BigInt::from(45))); }
    #[test] fn test_show_off_2_1_3_4_5_6_7_8_9() { let res = evaluate_expression("2(1/3 + 4(5/6 - 7/8)) / 9").unwrap(); assert_eq!(res, BigRational::new(BigInt::from(1), BigInt::from(27))); }
    #[test] fn test_show_off_neg3_2_5_1_4_3_1_2_7_8() { let res = evaluate_expression("-3(2/5 - 1/4(3 + 1/2)) + 7/8").unwrap(); assert_eq!(res, BigRational::new(BigInt::from(23), BigInt::from(10))); }
    #[test] fn test_new_edge_case() { let res = evaluate_expression("(1/3)/(2/5) + (4/6)*(7/8)/(9/10)").unwrap(); assert_eq!(res, BigRational::new(BigInt::from(40), BigInt::from(27))); }
    #[test] fn test_new_edge_case2() { let res = evaluate_expression("(1/4 + 1/5)/6 * 7/(2/3 - 1/8)").unwrap(); assert_eq!(res, BigRational::new(BigInt::from(63), BigInt::from(65))); }

    #[test] fn test_decimals() {
        let res = evaluate_expression("1.5 + 2/3").unwrap();
        assert_eq!(res, BigRational::new(BigInt::from(13), BigInt::from(6)));
        let res2 = evaluate_expression("0.25 * 4").unwrap();
        assert_eq!(res2, BigRational::from_integer(BigInt::from(1)));
        let res3 = evaluate_expression(".5 * 2").unwrap();
        assert_eq!(res3, BigRational::from_integer(BigInt::from(1)));
    }

    #[test] fn test_edge_1_plus_2_times_3_times_4_plus_5() { let res = evaluate_expression("(1+2)3(4+5)").unwrap(); assert_eq!(res, BigRational::from_integer(BigInt::from(81))); }
    #[test] fn test_edge_1_plus_2_times_3_plus_4_times_5_plus_6() { let res = evaluate_expression("(1+2)(3+4)(5+6)").unwrap(); assert_eq!(res, BigRational::from_integer(BigInt::from(231))); }
    #[test] fn test_edge_1_minus_nested() { let res = evaluate_expression("1 - (2 - (3 - 4))").unwrap(); assert_eq!(res, BigRational::from_integer(BigInt::from(-2))); }
    #[test] fn test_edge_nested_division() { let res = evaluate_expression("(1/(2/(3/4)))").unwrap(); assert_eq!(res, BigRational::new(BigInt::from(3), BigInt::from(8))); }
}