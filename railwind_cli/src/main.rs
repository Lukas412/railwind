use clap::Parser;
use config::Config;
use globwalk::glob;
use itertools::Itertools;
use notify::event::ModifyKind;
use notify::{Error, Event, EventKind, RecursiveMode, Watcher};
use railwind::{parse_to_string, CollectionOptions, Source, SourceOptions};
use ron::ser::PrettyConfig;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

mod config;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the output file
    #[arg(short, long, default_value = "railwind.css")]
    output: String,

    /// Include the Tailwind preflight in the output file
    #[arg(short = 'p', long, default_value = "false")]
    include_preflight: bool,

    /// Path to the config file
    #[arg(short = 'c', long, default_value = "railwind.config.ron")]
    config: String,

    /// Watch files for changes and automaticaly run the parser
    #[arg(short = 'w', long, default_value = "false")]
    watch: bool,

    /// Generate a default config file at the current directory
    #[arg(short = 'g', long, default_value = "false")]
    generate: bool,
}

fn main() {
    let args = Args::parse();

    if args.generate {
        let pretty = PrettyConfig::new().depth_limit(4);
        let config = ron::ser::to_string_pretty(&Config::default(), pretty).unwrap();
        let mut file = File::create("railwind.config.ron").unwrap();
        file.write_all(config.as_bytes()).unwrap();
        return;
    }

    let config = parse_config(&args.config);
    let input: Vec<_> = get_paths_from_config(&config).collect();
    let output = Path::new(&args.output);

    if args.watch {
        let mut watcher = notify::recommended_watcher(|res: Result<Event, Error>| match res {
            Ok(event) => match event.kind {
                EventKind::Modify(m) => match m {
                    ModifyKind::Data(_) => {
                        println!("Running parser");
                        let start = Instant::now();

                        let args = Args::parse();
                        let config = parse_config(&args.config);
                        let input = get_paths_from_config(&config).collect_vec();
                        let output = Path::new(&args.output);

                        run_parsing(&args, input.iter(), output, &config);

                        let duration = start.elapsed();
                        println!("Parsing took: {:?}", duration);
                    }
                    _ => (),
                },
                _ => (),
            },
            Err(e) => panic!("{}", e),
        })
        .unwrap();

        for watch_path in input.iter() {
            watcher
                .watch(watch_path, RecursiveMode::NonRecursive)
                .unwrap();
        }

        run_parsing(&args, input.iter(), output, &config);

        loop {}
    } else {
        run_parsing(&args, input.iter(), output, &config);
    }
}

fn parse_config(config_path: &str) -> Config {
    let config_file = match fs::read_to_string(config_path) {
        Ok(config_file) => config_file,
        Err(error) => {
            println!("Could not read config: {}. Using default config.", error);
            return Config::default();
        }
    };

    ron::from_str::<Config>(&config_file).unwrap_or_else(|error| {
        println!(
            "Failed to parse config: {}. Running with default config",
            error
        );
        Config::default()
    })
}

fn get_paths_from_config<'a>(config: &'a Config) -> impl Iterator<Item = PathBuf> + 'a {
    let content = config
        .content
        .iter()
        .map(Path::new)
        .filter(|path| path.is_dir())
        .filter_map(|path| fs::read_dir(path).ok())
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| entry.path());
    let glob_content = config
        .content
        .iter()
        .flat_map(|content| glob(content).unwrap())
        .filter_map(Result::ok)
        .map(|entry| entry.into_path());

    content.chain(glob_content)
}

fn run_parsing<'a>(
    args: &Args,
    input: impl IntoIterator<Item = &'a PathBuf>,
    output: &Path,
    config: &Config,
) {
    let mut warnings = vec![];

    let source_options: Vec<SourceOptions> = input
        .into_iter()
        .map(|i| SourceOptions {
            input: &i,
            option: if let Some(extension) = i.extension() {
                if let Some(str) = extension.to_str() {
                    if let Some(options) = &config.extend_collection_options {
                        CollectionOptions::new_expand(str, options)
                    } else {
                        CollectionOptions::new(str)
                    }
                } else {
                    CollectionOptions::String
                }
            } else {
                CollectionOptions::String
            },
        })
        .collect();

    let css = parse_to_string(
        Source::Files(source_options),
        args.include_preflight,
        &mut warnings,
    );

    let mut css_file = File::create(output).unwrap();
    css_file.write_all(css.as_bytes()).unwrap();

    for warning in warnings {
        println!("{}", warning)
    }
}
