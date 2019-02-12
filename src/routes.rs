use crate::commands::BunnyCommandTable;
use actix_web::{http, HttpRequest, HttpResponse};
use failure::Fallible;
use handlebars::{Handlebars, StringWriter};
use serde::Serialize;
use serde_json::json;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

type BunnyRequest = HttpRequest<BunnyCommandTable>;

/// Open a file from the resources directory
fn get_resource<T: AsRef<Path>>(
    resource: T,
    req: &BunnyRequest,
) -> Fallible<File> {
    Ok(File::open(
        req.state().resources_path.join(resource.as_ref()),
    )?)
}
/// Render a template as a pretty HTML page with bootstrap
fn render_template<Ctx, TemplatePath>(
    template: TemplatePath,
    context: &Ctx,
    req: &BunnyRequest,
) -> Fallible<String>
where
    TemplatePath: AsRef<Path>,
    Ctx: Serialize,
{
    let hb = Handlebars::new();
    let mut writer = StringWriter::new();

    let mut metadata_context: BTreeMap<&str, &str> = BTreeMap::new();
    metadata_context.insert("app_name", env!("CARGO_PKG_NAME"));
    metadata_context.insert("app_version", env!("CARGO_PKG_VERSION"));
    metadata_context.insert("app_authors", env!("CARGO_PKG_AUTHORS"));
    metadata_context.insert("route", req.path());
    metadata_context.insert("server_fqdn", &req.state().fqdn);

    writeln!(&mut writer, "<html>")?;
    hb.render_template_source_to_write(
        &mut get_resource("header.hbs", req)?,
        &metadata_context,
        &mut writer,
    )?;
    hb.render_template_source_to_write(
        &mut get_resource(template.as_ref(), req)?,
        context,
        &mut writer,
    )?;
    hb.render_template_source_to_write(
        &mut get_resource("footer.hbs", req)?,
        &metadata_context,
        &mut writer,
    )?;
    writeln!(&mut writer, "</html>")?;

    Ok(writer.into_string())
}
/// Render a template file without bootstrappy headers/footers
fn render_template_raw<Ctx, TemplatePath>(
    template: TemplatePath,
    context: &Ctx,
    req: &BunnyRequest,
) -> Fallible<String>
where
    TemplatePath: AsRef<Path>,
    Ctx: Serialize,
{
    let hb = Handlebars::new();
    let mut writer = StringWriter::new();

    hb.render_template_source_to_write(
        &mut get_resource(template.as_ref(), req)?,
        context,
        &mut writer,
    )?;

    Ok(writer.into_string())
}

/// Not-found page for routes
pub fn error_404(req: &BunnyRequest) -> Fallible<HttpResponse> {
    log::info!("404 not found: {}", req.path());
    let err_body = format!("404 for route `{}`", req.path());
    Ok(HttpResponse::build(http::StatusCode::NOT_FOUND)
        .content_type("text/html")
        .body(err_body))
}

/// Not-found page for commands
pub fn cmd_404(req: &BunnyRequest) -> Fallible<HttpResponse> {
    log::info!("404 not found: {}", req.path());
    let err_body = "No matching command";
    Ok(HttpResponse::build(http::StatusCode::NOT_FOUND)
        .content_type("text/html")
        .body(err_body))
}

/// Provide some metadata about turbobunny on the landing page
pub fn index(req: &BunnyRequest) -> Fallible<HttpResponse> {
    let body = render_template("about.hbs", &"", req)?;
    let resp = HttpResponse::build(http::StatusCode::OK)
        .content_encoding(actix_web::http::ContentEncoding::Auto)
        .content_type("text/html")
        .body(body);
    Ok(resp)
}

/// Render a list of all available commands
pub fn list(req: &BunnyRequest) -> Fallible<HttpResponse> {
    let body = render_template("list.hbs", &req.state().commands, req)?;
    let resp = HttpResponse::build(http::StatusCode::OK)
        .content_type("text/html")
        .body(body);
    Ok(resp)
}

/// Provide metadata for typeahead completions on matchers
pub fn suggest(req: &BunnyRequest) -> Fallible<HttpResponse> {
    let cmd_table = req.state();
    let resp;
    if let Some(query) = req.query().get("q") {
        let suggested_cmds = cmd_table.completions(&query);
        let suggestions: Vec<&str> =
            suggested_cmds.iter().map(|(_, matcher)| *matcher).collect();
        let descriptions: Vec<&str> = suggested_cmds
            .iter()
            .map(|(cmd, _)| cmd.description.as_str())
            .collect();
        let urls: Vec<String> = suggested_cmds
            .iter()
            .map(|(_, matcher)| {
                format!("https://{}/cmd?q={}", cmd_table.fqdn, matcher)
            })
            .collect();
        resp = json!([&query, suggestions, descriptions, urls]);
    } else {
        log::error!("suggest failed: missing query string (urlparam `q`)");
        resp = json!([]);
    }
    Ok(HttpResponse::build(http::StatusCode::OK)
        .content_type("application/json")
        .json(resp))
}

/// Yield an OpenSearch XML spec defining turbobunny as a browser search engine
pub fn search_xml(req: &BunnyRequest) -> Fallible<HttpResponse> {
    let mut context = BTreeMap::new();
    context.insert("server_fqdn", &req.state().fqdn);
    let body = render_template_raw("search.xml", &context, req)?;
    let resp = HttpResponse::build(http::StatusCode::OK)
        .content_type("application/xml")
        .body(body);
    Ok(resp)
}

/// Check whether a given string matches a command, and print info on the
/// match if one is found
pub fn check_cmd(req: &BunnyRequest) -> Fallible<HttpResponse> {
    let cmd_table = req.state();
    if let Some(query) = req.query().get("q") {
        if let Some((cmd, _)) = cmd_table.match_query(&query) {
            let body = render_template("check.hbs", cmd, req)?;
            let resp = HttpResponse::build(http::StatusCode::OK)
                .content_type("text/html")
                .body(body);
            return Ok(resp);
        }
    }
    cmd_404(req)
}

/// Run a command
pub fn cmd(req: &BunnyRequest) -> Fallible<HttpResponse> {
    let cmd_table = req.state();
    if let Some(query) = req.query().get("q") {
        if let Some((cmd, leftover_args)) = cmd_table.match_query(&query) {
            let resp = HttpResponse::Found()
                .header(http::header::LOCATION, cmd.run(leftover_args))
                .finish();
            return Ok(resp);
        }
        if let Some(fallback) = &cmd_table.fallback {
            let resp = HttpResponse::Found()
                .header(http::header::LOCATION, fallback.run(&query))
                .finish();
            return Ok(resp);
        }
    }
    error_404(req)
}
