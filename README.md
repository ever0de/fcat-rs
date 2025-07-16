# fcat

`fcat` is a command-line utility that recursively scans a directory or processes a single file and consolidates the contents of all found files into a single text file or directly to your clipboard.

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
fcat [OPTIONS] <PATH>
```

### Arguments

| Argument | Description                                                                                              |
| -------- | -------------------------------------------------------------------------------------------------------- |
| `<PATH>` | The path to the target directory or file. If a directory, scans recursively. If a file, processes only it. |

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

Just paste (`Ctrl+V` or `Cmd+V`) into your LLM prompt. No intermediate file needed.

```bash
fcat . --clipboard
```

*Short version:*

```bash
fcat . -c
```

**2. Generate an Output File**

If you prefer to save the context to a file, run `fcat .`. This will create a `bundler.txt` file in your current directory.

```bash
fcat .
```

**3. Bundle a Specific Directory or File**

Bundle all files from the `src` directory and save the result to `source_code.txt`:

```bash
fcat ./src -o source_code.txt
```

Or, bundle only a single file:

```bash
fcat src/main.rs -o main_only.txt
```

**4. Exclude an Additional Directory**

Bundle the project but also ignore the `docs` folder, in addition to the default ignores.

```bash
fcat . -c --exclude-dir ./docs
```

**5. Include Everything**

Bundle *all* files in the project, including the contents of `.git` and `target`, and copy it to the clipboard.

```bash
fcat . -c --no-default-ignores
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
