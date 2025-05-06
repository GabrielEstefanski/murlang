# Murlang - The Murloc Programming Language

**MRGLGLGLGL!** Welcome to Murlang, the programming language inspired by the Murlocs from the Warcraft universe!

## About

Murlang is an interpreted programming language with syntax based on "Murloc-speak". It incorporates modern programming concepts while maintaining the unique charm of Murloc communication:

- Asynchronous Programming (`mrglasync`, `mrglawait`)
- Parallel Programming with Threads (`splurg`, `mrglwait`)
- Higher-Order Functions
- Complex Data Structures
- Type System
- Error Handling with Murloc-themed messages

## Quick Start

```rust
// Variable declaration
grrr name = "Mrglglgl"
grrr age = 42

// Function definition
grrrfnrrg greet(name)
mrgl
    glglrr "Hello, " + name + "!"
grl

// Struct definition
rrkgr Murloc
mrgl
    nome: blbtxt,    // text type
    nivel: numblrr,  // number type
    vida: numblrr,   // number type
grl

// Control structures
grlbrr (age > 30)    // if
mrgl
    glglrr name + " is a wise murloc!"
grl
blrrgl              // else
mrgl
    glglrr name + " is a young murloc!"
grl

// Loops
gglrbl (age > 40)    // while
mrgl
    glglrr "Countdown: " + age
    age = age - 1
grl

// Function call
grrrblbl greet(name)

// Struct instance
grrr murloc_chief = Murloc { nome: "Grrmrgl", nivel: 10, vida: 100 }
```

## Language Features

### Keywords

| Murloc-Speak | Meaning | Example |
|--------------|---------|---------|
| `grrr` | Variable declaration | `grrr x = 10` |
| `grlbrr` | If statement | `grlbrr (x > 0)` |
| `grrrfnrrg` | Function definition | `grrrfnrrg sum(a, b)` |
| `grrrblbl` | Function call | `grrrblbl sum(5, 3)` |
| `mrgl` | Block start | `mrgl` |
| `grl` | Block end | `grl` |
| `glglrr` | Print | `glglrr "Hello"` |
| `grrrtn` | Return | `grrrtn result` |
| `splurg` | Spawn thread | `splurg { ... }` |
| `mrglwait` | Wait for thread | `mrglwait [thread1, thread2]` |
| `mrglasync` | Async function | `mrglasync fn task()` |
| `mrglawait` | Await operation | `mrglawait future` |

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
mrgl help      # Show available commands
```

## Building from Source

Murlang is built with Rust. To compile:

```bash
cargo build --release
```

The executable will be generated in `target/release/mur_lang`.

## Project Structure

```
mur_lang/
├── src/
│   ├── lexer.rs      # Tokenization
│   ├── parser.rs     # Syntax analysis
│   ├── ast.rs        # Abstract Syntax Tree
│   ├── interpreter/  # Runtime and execution
│   └── main.rs       # Entry point
├── scripts/          # Installation scripts
├── examples/         # Example programs
└── tests/           # Test suite
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

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by the Murlocs from World of Warcraft
- Built with Rust's amazing ecosystem
- Thanks to all contributors who make Murlang more Murloc-like!

## MRGLGLGLGL!

Join us in making Murlang the most Murloc-friendly programming language ever! Aaaaaughibbrgubugbugrguburgle! 