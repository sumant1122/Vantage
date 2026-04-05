<div align="center">

# 🐚 Vantage 

**The Performance-Focused, Environment-Aware Shell**

![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)
![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)
![Build](https://img.shields.io/badge/Build-Passing-brightgreen.svg)

<br>

*(Insert animated GIF of your glowing prompt and benchmarks here)*  
`![Vantage Demo](./assets/demo.gif)`

</div>

---

Vantage is an active shell written natively in Rust. It tracks your system health and benchmarks your workflows in real-time, proactively alerting you before your PC struggles with high-load workloads.

### ✨ Zero-Config Features

- 🚦 **Active Edge Dashboard**: Memory & CPU status are calculated directly inside your prompt without any rendering delay.
- 🦀 **Semantic Context**: Entering a project dynamically spotlights your stack (e.g. `Rust` or `Node.js`) and current `Git` branch.
- ⚡ **Ultra-Fast Native Tokenizer**: High-performance pipeline parsing guarantees `$PATH` autocompletion happens exactly when you hit `<TAB>`. 
- ⏱️ **Flight Recorder**: Prefix operations with `bench` (e.g., `bench cargo build`) to persistently log performance histories and receive alerts on regressions. 

### 📥 Install & Go

```bash
cargo build --release
./target/release/Vantage
```

### ⌨️ Quick Menu

Combine standard Bash-like pipelines (`|`, `<`, `>`) alongside our precision built-ins:

```bash
❯ sys                             # Output a full hardware/kernel report right to terminal
❯ top                             # Print the top 10 heavy CPU processes
❯ history                         # Query the latency reports of your `bench` actions
❯ ls -la | grep "rs" > output.txt # Leverage ultra-fast, robust internal pipelines
❯ cd my_rust_project              # The prompt automatically updates to standard POSIX contexts
```

---

<details>
<summary><b>🛠 Project Architecture</b></summary>
<br>

- `src/main.rs`: Prompt Engine 
- `src/parser.rs`: Custom zero-delay lexer/tokenizer
- `src/builtins.rs`: Fast OS interventions
- `src/monitor.rs`: Semantic system context polling
- `src/executor.rs`: Stdin/Stdout pipeline bridges
- `src/state.rs`: Non-blocking, XDG-standard persistent histories

</details>

*For contributing guidelines or code of conduct, please reference our [Contributing](CONTRIBUTING.md) and [License](LICENSE) files.*
