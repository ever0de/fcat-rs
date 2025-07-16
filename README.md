# fcat

`fcat` is a command-line utility that recursively scans a directory and consolidates the contents of all found files into a single text file.

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

2.  Build the release binary:
    ```bash
    cargo install --path .
    ```

## Usage

The basic command structure is simple:

```bash
fcat [OPTIONS]
```

### Options

| Option                 | Short | Description                                                        | Default       |
| ---------------------- | ----- | ------------------------------------------------------------------ | ------------- |
| `--target-dir`         | `-t`  | The target directory to scan.                                      | `.`           |
| `--output-file`        | `-o`  | The path for the output file.                                      | `bundler.txt` |
| `--exclude-dir`        | `-e`  | A specific directory to exclude from the scan.                     | (none)        |
| `--no-default-ignores` |       | Disables the default ignore list (includes `.git`, `target`, etc.). | (disabled)    |
| `--help`               | `-h`  | Print help information.                                            |               |
| `--version`            | `-V`  | Print version information.                                         |               |

### Examples

1. Generate AI Context for a Project

This is the most common use case. Run this command in your project's root directory. It will create a `bundler.txt` file. Open it, copy the entire contents, and paste it into your LLM prompt.

```bash
fcat
```

2. Specify a Target and Output

Bundle all files from the `src` directory and save the result to `source_code.txt`.

```bash
fcat --target-dir ./src --output-file source_code.txt
```

*Short version:*

```bash
fcat -t ./src -o source_code.txt
```

3. Exclude an Additional Directory

Bundle the project but also ignore the `docs` folder.

```bash
fcat --exclude-dir ./docs
```

4. Include Everything

Bundle *all* files in the project, including the contents of `.git` and `target`, by disabling the default ignores.

```bash
fcat --no-default-ignores
```

## Output Format

`fcat` wraps the content of each file with a simple header and footer indicating the file's path. This makes the resulting bundle easy to read and parse.

**Example `bundler.txt`:**

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
