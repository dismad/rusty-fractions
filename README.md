# rusty-fractions
Operate on fractions using RUST

![frac](https://github.com/user-attachments/assets/d2c29ee6-c463-4889-873e-77af9be53c17)


![cf](https://github.com/user-attachments/assets/e2161bf6-1e9c-42da-85e8-a5e0e2186467)



---

## Features

- Exact arithmetic using BigRational (e.g. 1/3 + 1/5 = 8/15)
- Full decimal support (1.5, 0.25, .5, etc.)
- Implied multiplication (2(3+4), (1+2)3(4+5))
- Left-associative chained divisions (1/2/3/4 = 1/24) with clear warning for 3 or more divisions
- Continued Fractions
- Step-by-step calculation with clean rendered PNGs (no duplicate steps)
- Beautiful math rendering via Typst
- Export options: final_render.png, steps_render.png, math_output.md, and math_output.tex
- Modern, responsive GUI built with egui/eframe
- Background processing keeps the interface responsive

---

## Installation

### Prerequisites
- Rust toolchain (latest stable)
- Typst CLI (required for math rendering)

```bash
# macOS
brew install typst

# Windows (via Scoop)
scoop install typst

# Linux
cargo install typst-cli
```

## Build from source

```
git clone https://github.com/dismad/rusty_fractions.git
cd rusty_fractions
cargo build --release
```


## How to Use

 - Type any fraction expression in the large input field. Click Convert.
 - (Optional) Check Show step-by-step calculation to see detailed steps and rendered images.
 - Click Save Renders to export the PNG files plus Markdown and LaTeX versions.

## Example Expressions

| Expression                              | Result     | Notes                                      |
|-----------------------------------------|------------|--------------------------------------------|
| `1.5 + 2/3`                             | `13/6`     | Decimal + fraction                         |
| `(1+2)3(4+5)`                           | `81`       | Implied multiplication                     |
| `(1+2)(3+4)(5+6)`                       | `231`      | Multiple implied multiplications           |
| `1 - (2 - (3 - 4))`                     | `-2`       | Nested parentheses                         |
| `(1/(2/(3/4)))`                         | `3/8`      | Nested division                            |
| `1/3/(2/5) + 4/6*7/8/9/10`              | (with warning) | Ambiguity warning for chained divisions |


## Technical Details

 - GUI: egui + eframe
 - Exact math: num-bigint + num-rational
 - Rendering: Typst CLI (via subprocess)
 - Image handling: image crate
 - Threading: Background evaluation
