use clap::{
    Args, Parser, Subcommand
};
use std::{
    fs, env,
    io::Read, 
    process::Command
};
use yaml_rust::{
    YamlLoader, YamlEmitter
};

use curl::easy::Easy;
use http::Uri;

const OAI_SPEC: &str = include_str!("../unit-openapi.yaml");

#[derive(Parser)]
#[command(name = "unitctl")]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Start(StartArgs),

    // TODO : Modify status call to point to some specific API endpoint    
    #[command(arg_required_else_help = true)]
    Status(StatusArgs),
    
    // TODO : Validate calls with openapi spec
    // TODO : Access over SSH
    // TODO : Access within docker containers
    // TODO : Support custom certs probably
    // TODO : Turn on netrc support probably?
    #[command(arg_required_else_help = true)]
    API(APIArgs),
    
    // TODO : Load spec from submodule instead of duplicate file
    // TODO : Use an actual OpenAPI spec parser
    #[command(arg_required_else_help = true)]
    Schema(SchemaArgs),
}

#[derive(Args, Clone)]
struct StartArgs {
    #[arg(
        short, long,
        required = true,
        help = "path to desired control socket"
    )]
    socket: String
}

#[derive(Args, Clone)]
struct StatusArgs {
    #[arg(
        short, long,
        required = true,
        help = "URI the control API listens on"
    )]
    uri: String,
    
    #[arg(
        short, long,
        help = "Unix Socket the control API listens on"
    )]
    socket: Option<String>,
    
    #[arg(
        short, long,
        help = "switch to trigger verbose behavior in libcurl"
    )]
    verbose: bool,
}

#[derive(Args, Clone)]
struct APIArgs {
    #[arg(
        short, long, 
        required = true,
        help = "URI for API operation"
    )]
    uri: String,

    #[arg(
        short, long,
        help = "Unix Socket the control API listens on"
    )]
    socket: Option<String>,
    
    #[arg(
        short, long, 
        conflicts_with = "file",
        help = "inline JSON data to post to API"
    )]
    json: Option<String>,
       
    #[arg(
        short, long,
        help = "file containing data to post to API."
    )]
    file: Option<String>,
       
    #[arg(
        short, long,
        help = "switch to trigger a delete operation on an API endpoint.",
        conflicts_with_all = ["file", "json"]
    )]
    delete: bool,

    #[arg(
        short, long,
        help = "switch to trigger verbose behavior in libcurl"
    )]
    verbose: bool,
}

#[derive(Args, Clone)]
struct SchemaArgs {
    #[arg(
        short, long,
        required = true,
        help = "path for schema query"
    )]
    path: String
}

fn do_start(args: StartArgs) {
    Command::new("docker")
        .args(["pull", "unit"])
        .spawn()
        .expect("failed to call Docker")
        .wait()
        .expect("expected Docker to succeed");
        
    let appmount = format!(
        "type=bind,src={},dst=/www", 
        env::current_dir().unwrap().display()
        );
    
    let sockmount = format!(
        "type=bind,src={},dst=/var/run/",
        args.socket
        );
    
    Command::new("docker")
        .args(["run", "-d", 
            "--mount", appmount.as_str(), 
            "--mount", sockmount.as_str(),
            "--network", "host", "unit"])
        .spawn()
        .expect("failed to call Docker")
        .wait()
        .expect("expected Docker to succeed");
        
    println!("Congratulations! NGINX Unit now running at {}/control.unit.sock", args.socket);
    println!("NOTICE: Socket access is root only by default. Run chown.");
    println!("Current directory mounted to /www in NGINX Unit container.");
}

fn get_status(args: StatusArgs) {
    do_api_call(APIArgs {
        socket: args.socket,
        uri: args.uri,
        verbose: args.verbose,
        json: None, file: None, 
        delete: false,
    })
}

fn do_api_call(args: APIArgs) {
    let mut curl = Easy::new();

    if let Some(path) = args.socket {
        curl.unix_socket(path.as_str()).unwrap();
    } 
    curl.url(args.uri.as_str()).unwrap();
    curl.verbose(args.verbose).unwrap();
    
    let contents: Option<String>;
    if let Some(path) = args.file {
        contents = Some(fs::read_to_string(path)
            .expect("Should have been able to read the file"));
    } else {
        contents = args.json;
    }
    
    if let None = contents {
        curl.get(true).unwrap();
        if let Err(e) = curl.perform() {
            eprintln!("error in API call: {}", e)
        }
    } else {
        curl.post(true).unwrap();
        curl.post_field_size(contents.clone().unwrap().len() as u64).unwrap();
        let mut transfer = curl.transfer();
        transfer.read_function(|buf| {
            Ok(contents.clone().unwrap().as_bytes().read(buf).unwrap_or(0))
        }).unwrap();
        if let Err(e) = transfer.perform() {
            eprintln!("error in API call: {}", e)
        }
    }
}

fn get_schema(args: SchemaArgs) {
    let maybe_path = args.path.parse::<Uri>();
    if let Err(e) = maybe_path {
        eprintln!("Error: couldn't load path from uri: {}", e);
        return
    }
    let path = maybe_path.unwrap();
    
    let spec = YamlLoader::load_from_str(OAI_SPEC).unwrap();
    if spec[0]["paths"].is_badvalue() {
        eprintln!("Error: no paths in OpenAPI spec!")
    }
    if spec[0]["paths"][path.path()].is_badvalue() {
        eprintln!("Error: requested path not found.");
        eprintln!("\tConsider checking manually:");
        eprintln!("\thttps://github.com/nginx/unit/blob/master/docs/unit-openapi.yaml");
        return;
    }
    
    let pathspec = spec[0]["paths"][path.path()].clone();
    let mut out_str = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter.dump(&pathspec).unwrap(); // dump the YAML object to a String
    }
    
    println!("{}", out_str);        
}

fn main() {
    let call = Cli::parse();
    match call.command {
        Commands::Start(args) => do_start(args),
        Commands::Status(args) => get_status(args),
        Commands::API(args) => do_api_call(args),
        Commands::Schema(args) => get_schema(args),
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert()
}