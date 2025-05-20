# Murlang - The Murloc Programming Language

**MRGLGLGLGL!** Welcome to Murlang, the programming language inspired by the Murlocs from the Warcraft universe!

## About

Murlang is an interpreted programming language with syntax based on "Murloc-speak". It incorporates modern programming concepts while maintaining the unique charm of Murloc communication:

- Asynchronous Programming (`argl`, `mrgargl`)
- Parallel Programming with Threads (`splurg`, `mrgurl`)
- Higher-Order Functions
- Complex Data Structures
- Type System
- Error Handling with Murloc-themed messages

## Quick Start (Basic Syntax)

```murlang
// Declaração de variável
grrr name = "Mrglglgl"
grrr age = 42

// Definição de função
grrrfnrrg greet(name)
mrgl
    glglrr "Hello, " + name + "!"
grl

// Definição de struct
rrkgr Murloc
mrgl
    name: blbtxt,    // tipo texto
    level: numblrr,  // tipo número
    health: numblrr, // tipo número
grl

// Estruturas de controle
grlbrr (age > 30)    // if
mrgl
    glglrr name + " is a wise murloc!"
grl
blrrgl              // else
mrgl
    glglrr name + " is a young murloc!"
grl

gglrbl (age > 40)   // while
mrgl
    glglrr "Countdown: " + age
    age = age - 1
grl

grrrblbl greet(name)

// Instanciando struct
grrr murloc_chief = Murloc { name: "Grrmrgl", level: 10, health: 100 }
```

See [examples/run.mur](examples/run.mur) for a complete program.

## Language Features

### Keywords

| Murloc-Speak | Meaning | Example |
|--------------|---------|---------|
| `grrr` | Variable declaration | `grrr x = 10` |
| `grlbrr` | If statement | `grlbrr (x > 0)` |
| `blrrgl` | Else | `blrrgl` |
| `grrrfnrrg` | Function definition | `grrrfnrrg sum(a, b)` |
| `grrrblbl` | Function call | `grrrblbl sum(5, 3)` |
| `mrgl` | Block start | `mrgl` |
| `grl` | Block end | `grl` |
| `glglrr` | Print | `glglrr "Hello"` |
| `grrrtn` | Return | `grrrtn result` |
| `splurg` | Spawn thread | `splurg` |
| `mrgurl` | Wait for thread | `mrgurl` |
| `argl` | Async function| `argl grrrfnrrg task()` |
| `mrgargl` | Await Operation | `mrgargl future` |
| `rrkgr` | Struct | `rrkgr Person` |
| `grrip` | Array | `grrip numbers = [1,2,3]` |
| `gglrbl` | While loop | `gglrbl (cond)` |
| `mrrg` | For loop | `mrrg (item in list)` |
| `murrrgh` | Switch | `murrrgh (var)` |
| `grlblgl` | Case | `grlblgl 1:` |
| `blrrghlt` | Default | `blrrghlt:` |
| `blgr` | In | `mrrg member blgr clan` |
| `mrglgl` | Try | `mrglgl` |
| `mrglurp` | Catch | `mrglurp` |

### Data Types

- Numbers (`numblrr`)
- Text (`blbtxt`)
- Arrays (`grrip`)
- Structs (`rrkgr`)
- Threads
- Futures

### Control Structures

- If/Else (`grlbrr`/`blrrgl`)
- While loops (`gglrbl`)
- For loops (`mrrg`)
- Switch statements (`murrrgh`)
- Try/Catch (`mrglswim`/`mrglcatch`)

## Installation

### Windows

```batch
cd scripts
install.bat
```

Add the bin directory to your PATH or run `bin\activate.bat` to set up the environment.

### Linux/macOS

```bash
cd scripts
chmod +x install.sh
./install.sh
```

Add the bin directory to your PATH or run `source bin/activate` to set up the environment.

## Usage

After installation, you can run Murlang programs using:

```bash
mrgl run my_program.mur
```

### Additional Commands

```bash
mrgl version   # Show Murlang version
mrgl help      # List all available commands
```

## Building from Source

Murlang is implemented in Rust. To compile:

```bash
cargo build --release
```

The executable will be generated in `target/release/mur_lang`.

## Project Structure

```
mur_lang/
├── bin/                  # Executables or compiled binaries
├── examples/             # Example programs
├── scripts/              # Installation scripts
├── src/
│   ├── interpreter/      # Runtime and execution logic
│   ├── ast.rs            # Abstract Syntax Tree definitions
│   ├── expression_parser.rs  # Expression parser
│   ├── lexer.rs          # Tokenization
│   ├── lib.rs            # Main library entry (for `cargo build --lib`)
│   ├── main.rs           # Binary entry point
│   ├── mod.rs            # Root module for src
│   ├── parser.rs         # Syntax parser
│   └── value_parser.rs   # Parser for literals/values

```

## Contributing

MRGLGLGLGL! We welcome contributions! Here's how you can help:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Setup

1. Install Rust and Cargo
2. Clone the repository
3. Run tests: `cargo test`
4. Build: `cargo build`

### Code Style

- Follow Rust's standard formatting: `cargo fmt`
- Run clippy: `cargo clippy`
- Maintain the Murloc theme in error messages and documentation

## License

Murlang is licensed under the MIT License. See the full license [here](LICENSE).

## VSCode Extension

The official Murlang VSCode extension is available on the [Visual Studio Marketplace](https://marketplace.visualstudio.com/items?itemName=GabrielEstefanski.murlang).
Enhance your development experience with syntax highlighting, snippets, and more!

## Acknowledgments

- Inspired by the Murlocs from World of Warcraft
- Powered by Rust’s incredible ecosystem
- Thanks to all contributors who make Murlang more Murloc-like!

## MRGLGLGLGL!

Join us in making Murlang the most Murloc-friendly programming language ever! Aaaaaughibbrgubugbugrguburgle! 