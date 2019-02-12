use clap::{AppSettings, Arg};
use listenfd::ListenFd;

const DEFAULT_PORT: i64 = 8080;

#[macro_export]
macro_rules! gen_service_metadata_functions {
    () => (
        fn get_name(&self) -> &str { env!("CARGO_PKG_NAME") }
        fn get_version(&self) -> &str { env!("CARGO_PKG_VERSION") }
        fn get_authors(&self) -> &str { env!("CARGO_PKG_AUTHORS") }
        fn get_description(&self) -> &str { env!("CARGO_PKG_DESCRIPTION") }
    )
}

pub trait LudumIpsumService {
    type State;

    fn get_name(&self) -> &str;
    fn get_version(&self) -> &str;
    fn get_authors(&self) -> &str;
    fn get_description(&self) -> &str;

    /// Parse any service-specific command line arguments you need
    fn parse_args<'a, 'b>(&self, cli: clap::App<'a, 'b>) -> clap::App<'a, 'b>;

    /// Provide your actix-web app and we'll wrap it in a server object
    fn configure_server(
        &self,
        args: &clap::ArgMatches,
    ) -> actix_web::App<Self::State>;
}

pub fn run_server<Service>(service: Service)
where
    Service: LudumIpsumService + Send + Clone + 'static,
{
    let name = service.get_name().to_string();
    let version = service.get_version().to_string();
    let authors = service.get_authors().to_string();
    let description = service.get_description().to_string();

    log::info!("Parsing CLI arguments");
    let extended_description = format!(
        "{}\n\n \
         If you're working on this service actively, consider \
         using systemfd and cargo-watch for live-coding: \
         * cargo install systemfd cargo-watch \
         * systemfd --no-pid -s http::8080 -- cargo watch -x 'run --bin {}'",
        description, name
    );

    let cli = clap::App::new(name)
        .version(version.as_str())
        .author(authors.as_str())
        .about(extended_description.as_str())
        .setting(AppSettings::ColorAuto)
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::GlobalVersion)
        .setting(AppSettings::InferSubcommands)
        .setting(AppSettings::UnifiedHelpMessage)
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .takes_value(true)
                .help("listen port number, if not provided by listenfd"),
        )
        .arg(
            Arg::with_name("bindip")
                .short("b")
                .long("bindip")
                .takes_value(true)
                .help("bind to this IP, if not provided by listenfd"),
        );
    let cli = service.parse_args(cli);
    let options = cli.get_matches();

    log::info!("Configuring server instance");
    let server_options = options.clone();
    let mut server = actix_web::server::new(move || {
        // TODO: Configure common middleware
        service.configure_server(&server_options).finish()
    });

    log::info!("Checking for listen socket from systemd or systemfd");
    let mut listenfd = ListenFd::from_env();
    server = if let Ok(Some(l)) = listenfd.take_tcp_listener(0) {
        log::info!("Got one! Using system listener: {:#?}", &l);
        server.listen(l)
    } else {
        let bindip: &str = options.value_of("bindip").unwrap_or("127.0.0.1");
        let port: i64 = options
            .value_of("port")
            .unwrap_or(&format!("{}", DEFAULT_PORT))
            .parse()
            .unwrap_or(DEFAULT_PORT);
        let bindstr = format!("{}:{}", bindip, port);
        log::info!("No system fd found. Binding to {}", bindstr);
        server.bind(bindstr).unwrap()
    };

    log::info!("Running service!");
    server.run();
}
