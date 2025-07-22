use std::fmt::Write;
use std::path::PathBuf;
use std::sync::Arc;

use arboard::Clipboard;
use async_recursion::async_recursion;
use clap::Parser;
use globset::{Glob, GlobSet, GlobSetBuilder};
use once_cell::sync::Lazy;
use rlimit::Resource;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tokio::sync::{Semaphore, mpsc};
use tokio::task::JoinSet;
use tokio::time::Instant;

const CONCURRENT_TASKS: usize = 32768;
const DEFAULT_IGNORES: &[&str] = &[
    // fcat
    "**/bundler.txt",
    // license
    "**/LICENSE*",
    // git
    "**/.git/**",
    "**/.gitignore",
    // rust
    "**/target/**",
    "**/Cargo.lock",
    // node
    "**/node_modules/**",
    "**/dist/**",
];

static IGNORE_SET: Lazy<GlobSet> = Lazy::new(|| {
    let mut builder = GlobSetBuilder::new();
    for pattern in DEFAULT_IGNORES {
        builder.add(Glob::new(pattern).expect("Failed to compile glob pattern"));
    }
    builder.build().expect("Failed to build glob set")
});

/// A fast, asynchronous tool to recursively bundle file contents into a single text file.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The paths to the target directories or files.
    /// Supports multiple paths and shell-expanded glob patterns (e.g., `src/**/*.rs`).
    #[arg(required = true)]
    paths: Vec<PathBuf>,

    /// A specific directory path to exclude from the search.
    ///
    /// Any files or subdirectories within this path will be ignored, in addition to
    /// the default ignores.
    #[arg(short, long, value_name = "DIR")]
    exclude_dir: Option<PathBuf>,

    /// The path to the output file where all content will be bundled.
    /// This option is ignored when --clipboard is used.
    #[arg(short, long, default_value = "bundler.txt")]
    output_file: PathBuf,

    /// Disables the default ignore list (e.g., .git, target).
    ///
    /// By default, common development directories are ignored to prevent bundling
    /// unwanted files. Use this flag to include them in the search.
    #[arg(long)]
    no_default_ignores: bool,

    /// Copy the output directly to the clipboard instead of a file.
    #[arg(short, long, conflicts_with = "output_file")]
    clipboard: bool,
}

#[derive(Clone)]
struct AppContext {
    exclude_dir: Option<PathBuf>,
    no_default_ignores: bool,
    semaphore: Arc<Semaphore>,
}

type FileData = (PathBuf, String);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    rlimit::increase_nofile_limit(u64::MAX).expect("failed during increasing NOFILE rlimit");

    let nofile = rlimit::getrlimit(Resource::NOFILE).expect("cannot query rlimit");
    if nofile.0 < 1024 || nofile.1 < 1024 {
        eprintln!(
            "warning: NOFILE resource limit is low(={nofile:?}), run `ulimit -n 65536` and try again if panic occurs"
        );
    }

    let exclude_dir_abs = if let Some(dir) = args.exclude_dir {
        Some(fs::canonicalize(dir).await?)
    } else {
        None
    };

    let context = AppContext {
        exclude_dir: exclude_dir_abs,
        no_default_ignores: args.no_default_ignores,
        semaphore: Arc::new(Semaphore::new(CONCURRENT_TASKS)),
    };

    let (tx, mut rx) = mpsc::channel::<FileData>(CONCURRENT_TASKS);

    let start = Instant::now();

    let writer_task = if args.clipboard {
        tokio::spawn(async move {
            let mut full_content = String::new();
            while let Some((path, content)) = rx.recv().await {
                writeln!(&mut full_content, "<{}>", path.display())?;
                writeln!(&mut full_content, "{content}")?;
                writeln!(&mut full_content, "<{}/>\n\n", path.display())?;
            }
            let mut clipboard = Clipboard::new()?;
            clipboard.set_text(full_content)?;
            Ok::<(), eyre::Report>(())
        })
    } else {
        let output_file_path = args.output_file.clone();
        tokio::spawn(async move {
            let mut output_file = File::create(&output_file_path)
                .await
                .expect("Failed to create output file");

            while let Some((path, content)) = rx.recv().await {
                let header = format!("<{}>\n", path.display());
                let footer = format!("\n<{}/>\n\n", path.display());

                output_file.write_all(header.as_bytes()).await.unwrap();
                output_file.write_all(content.as_bytes()).await.unwrap();
                output_file.write_all(footer.as_bytes()).await.unwrap();
            }
            Ok::<(), eyre::Report>(())
        })
    };

    let mut reader_tasks = JoinSet::new();
    for path in args.paths {
        reader_tasks.spawn(process_path_recursively(context.clone(), path, tx.clone()));
    }

    while let Some(res) = reader_tasks.join_next().await {
        if let Err(e) = res {
            eprintln!("[Error] A reader task panicked: {e}");
        }
    }

    drop(tx);

    writer_task.await??;

    if args.clipboard {
        println!(
            "Successfully copied to clipboard. (took: {:.2?})",
            start.elapsed()
        );
    } else {
        println!(
            "Successfully created '{}'. (took: {:.2?})",
            args.output_file.display(),
            start.elapsed()
        );
    }

    Ok(())
}

#[async_recursion]
async fn process_path_recursively(
    ctx: AppContext,
    path: PathBuf,
    tx: mpsc::Sender<FileData>,
) -> eyre::Result<()> {
    if !ctx.no_default_ignores && IGNORE_SET.is_match(&path) {
        return Ok(());
    }

    if let Some(excluded) = &ctx.exclude_dir {
        if let Ok(current_abs) = fs::canonicalize(&path).await {
            if current_abs.starts_with(excluded) {
                return Ok(());
            }
        }
    }

    let _permit = ctx.semaphore.acquire().await.expect("Semaphore closed");

    let metadata = fs::metadata(&path).await.map_err(|e| {
        eprintln!(
            "[Error] Could not read metadata for {}: {}",
            path.display(),
            e
        );
        eyre::eyre!("Failed to read metadata for {}: {}", path.display(), e)
    })?;

    if metadata.is_dir() {
        let mut entries = fs::read_dir(path).await?;
        let mut tasks = JoinSet::new();

        while let Some(entry) = entries.next_entry().await? {
            tasks.spawn(process_path_recursively(
                ctx.clone(),
                entry.path(),
                tx.clone(),
            ));
        }

        while let Some(res) = tasks.join_next().await {
            if let Err(e) = res {
                eprintln!("[Error] A sub-task panicked: {e}");
            }
        }
    } else if metadata.is_file() {
        match fs::read_to_string(&path).await {
            Ok(content) => {
                if tx.send((path, content)).await.is_err() {
                    eprintln!("[Error] Failed to send file data to writer: channel closed.");
                }
            }
            Err(e) => {
                eprintln!("[Warning] Skipping file {}: {}", path.display(), e);
            }
        }
    }

    Ok(())
}
