use deadwiki::web;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<String>>();
    let mut args = args.iter();
    let mut path = "./wiki";
    let mut host = "0.0.0.0";
    let mut port = 8000;

    while let Some(arg) = args.next() {
        match arg.as_ref() {
            "-v" | "-version" | "--version" => return print_version(),
            "-h" | "-help" | "--help" => return print_help(),
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
            _ => path = arg,
        }
    }

    if let Err(e) = web::server(path, host, port) {
        eprintln!("WebServer Error: {}", e);
    }
}

fn print_version() {
    println!("deadwiki v{}", env!("CARGO_PKG_VERSION"))
}

fn print_help() {
    print!(
        "Usage: dead [options] <PATH TO WIKI>

Options:
    -H, --host     Host to bind to. Default: 0.0.0.0
    -p, --port     Port to bind to. Default: 8000
    -v, --version  Print version.
    -h, --help     Show this message.
"
    );
}
