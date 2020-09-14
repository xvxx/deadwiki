use deadwiki::{app, db, sync};

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<String>>();
    let mut args = args.iter();
    let mut path = "";
    let mut host = "0.0.0.0";
    let mut port = 8000;
    let mut sync = false;
    #[cfg(feature = "gui")]
    let mut gui = false;

    while let Some(arg) = args.next() {
        match arg.as_ref() {
            "-v" | "-version" | "--version" => return print_version(),
            "-h" | "-help" | "--help" => return print_help(),
            "-s" | "-sync" | "--sync" => sync = true,
            "-H" | "-host" | "--host" => {
                if let Some(arg) = args.next() {
                    host = arg;
                } else {
                    return eprintln!("--host needs a value");
                }
            }
            "-p" | "-port" | "--port" => {
                if let Some(arg) = args.next() {
                    port = arg.parse().unwrap();
                } else {
                    return eprintln!("--port needs a value");
                }
            }
            #[cfg(feature = "gui")]
            "-g" | "-gui" | "--gui" => gui = true,
            _ => {
                if arg.starts_with('-') {
                    return eprintln!("unknown option: {}", arg);
                }
                path = arg;
            }
        }
    }

    println!("~> deadwiki v{}", env!("CARGO_PKG_VERSION"));

    #[cfg(feature = "gui")]
    {
        if gui {
            if let Err(e) = deadwiki::gui::run(host, port, path, sync) {
                eprintln!("GUI Error: {}", e);
            }
            return;
        }
    }

    if path.is_empty() {
        return print_help();
    }

    if let Err(e) = deadwiki::set_wiki_root(path) {
        eprintln!("Wiki Error: {}", e);
        return;
    }

    if sync {
        if let Err(e) = sync::start() {
            eprintln!("Sync Error: {}", e);
            return;
        }
    }

    let db = db::DB::new(deadwiki::wiki_root());
    vial::use_state!(db);

    if let Err(e) = vial::run!(format!("{}:{}", host, port), app) {
        eprintln!("WebServer Error: {}", e);
    }
}

fn print_version() {
    println!("deadwiki v{}", env!("CARGO_PKG_VERSION"))
}

fn print_help() {
    let mut gui = "";
    if cfg!(feature = "gui") {
        gui = "    -g, --gui      Launch as WebView application.\n";
    }

    print!(
        "Usage: dead [options] <PATH TO WIKI>

Options:
    -H, --host     Host to bind to. Default: 0.0.0.0
    -p, --port     Port to bind to. Default: 8000
    -s, --sync     Automatically sync wiki. Must be a git repo.
{gui}
    -v, --version  Print version.
    -h, --help     Show this message.
",
        gui = gui
    );
}
