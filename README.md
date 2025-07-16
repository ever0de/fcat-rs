# fcat

`fcat` is a command-line utility that recursively scans directories, processes files, and consolidates the contents of all found files into a single text file or directly to your clipboard.

It's designed to make it easy to gather project context, create source code archives, or generate project overviews. Its primary use case is to quickly create a single context file that can be pasted into a prompt for a Large Language Model (LLM), providing the AI with the full context of a project.

## Installation

### Cargo

TODO:

### From Source

Alternatively, you can build it from the source code:

1.  Clone the repository:

    ```bash
    git clone https://github.com/ever0de/fcat.git
    cd fcat
    ```

2.  Build and install the binary:

    ```bash
    cargo install --path .
    ```

## Usage

The basic command structure is simple:

```bash
fcat [OPTIONS] <PATHS>...
```

### Arguments

| Argument | Description                                                                                              |
| -------- | -------------------------------------------------------------------------------------------------------- |
| `<PATHS>...` | One or more paths to target directories or files. Supports shell-expanded glob patterns (e.g., `src/**/*.rs`). |

### Options

| Option                 | Short | Description                                                        | Default       |
| ---------------------- | ----- | ------------------------------------------------------------------ | ------------- |
| `--output-file`        | `-o`  | The path for the output file.                                      | `bundler.txt` |
| `--clipboard`          | `-c`  | Copy the output directly to the clipboard instead of a file.       | (disabled)    |
| `--exclude-dir`        | `-e`  | A specific directory to exclude from the scan.                     | (none)        |
| `--no-default-ignores` |       | Disables the default ignore list (includes `.git`, `target`, etc.). | (disabled)    |
| `--help`               | `-h`  | Print help information.                                            |               |
| `--version`            | `-V`  | Print version information.                                         |               |

### Examples

**1. Copy Current Directory to Clipboard**

This is the most common use case. Run this command in your project's root directory. It will bundle the project's contents and copy the result directly to your clipboard.

```bash
fcat . -c
```

**2. Select Specific Files and Directories**

You can specify multiple paths. For example, to bundle all files in the `src` directory and the `Cargo.toml` file:

```bash
fcat src Cargo.toml -c
```

**3. Use Glob Patterns to Select Files**

Use your shell's globbing capabilities to select files with specific patterns. For example, to bundle all `.rs` files in the `src` directory and the `Cargo.toml` file:

```bash
fcat src/**/*.rs Cargo.toml -c
```

*(Note: Glob pattern behavior may vary slightly depending on your shell, e.g., zsh, bash.)*


**4. Generate an Output File**

If you prefer to save the context to a file, omit the `-c` or `--clipboard` flag. This command will create a `bundler.txt` file.

```bash
fcat .
```

**5. Exclude an Additional Directory**

Bundle the project but also ignore the `docs` folder, in addition to the default ignores.

```bash
fcat . -c --exclude-dir ./docs
```

## Output Format

`fcat` wraps the content of each file with a simple header and footer indicating the file's path. This makes the resulting bundle easy to read and parse.

**Example output:**

```text
<src/main.rs>
fn main() {
    println!("Hello from main!");
}
<src/main.rs/>

<src/lib.rs>
pub fn a_helpful_function() -> bool {
    true
}
<src/lib.rs/>
```
