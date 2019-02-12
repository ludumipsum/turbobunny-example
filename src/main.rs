mod service_base;

use service_base::LudumIpsumService;
use std::path::{Path, PathBuf};

mod commands;
mod routes;

#[derive(Clone)]
struct TurboBunnyService {
    resources_path: PathBuf,
}
impl TurboBunnyService {
    fn new<T: AsRef<Path>>(resources_path: T) -> Self {
        Self {
            resources_path: resources_path.as_ref().to_owned(),
        }
    }
}
impl LudumIpsumService for TurboBunnyService {
    type State = commands::BunnyCommandTable;

    gen_service_metadata_functions!();

    fn parse_args<'a, 'b>(&self, cli: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
        cli.arg(
            clap::Arg::with_name("fqdn")
                .help("fqdn where this service runs")
                .required(true),
        )
    }

    fn configure_server(
        &self,
        args: &clap::ArgMatches,
    ) -> actix_web::App<Self::State> {
        let fqdn: &str = args.value_of("fqdn").unwrap_or("0.0.0.0");
        let favicon_path = Box::new(self.resources_path.join("favicon.ico"));
        actix_web::App::with_state(commands::BunnyCommandTable::new(
            fqdn,
            &self.resources_path,
        ))
        .resource("/", |r| r.f(routes::index))
        .resource("/index", |r| r.f(routes::index))
        .resource("/index.htm", |r| r.f(routes::index))
        .resource("/index.html", |r| r.f(routes::index))
        .resource("/list", |r| r.f(routes::list))
        .resource("/cmd", |r| r.f(routes::cmd))
        .resource("/check", |r| r.f(routes::check_cmd))
        .resource("/suggest", |r| r.f(routes::suggest))
        .resource(r"/search.xml", |r| r.f(routes::search_xml))
        .resource(r"/favicon.ico", move |r| {
            r.f(move |_| actix_web::fs::NamedFile::open(&*favicon_path))
        })
        .handler(
            r"/static",
            actix_web::fs::StaticFiles::new(&self.resources_path)
                .expect("Failed to open static resources path"),
        )
        .default_resource(|r| {
            // 404 for GET request
            r.method(actix_web::http::Method::GET).f(routes::error_404);

            // all requests that are not `GET`
            r.route()
                .filter(actix_web::pred::Not(actix_web::pred::Get()))
                .f(|_req| actix_web::HttpResponse::MethodNotAllowed());
        })
    }
}

fn main() {
    let env = env_logger::Env::default().default_filter_or("info");
    env_logger::Builder::from_env(env)
        .default_format_timestamp_nanos(true)
        .init();
    log::trace!("configured global log handler");;

    let service = TurboBunnyService::new("./resources");
    service_base::run_server(service);
}
