use std::{env, thread};

use async_std::task;
use log::info;
use main_error::MainError;
use structopt::StructOpt;
use tide::middleware::{Cors, RequestLogger};

use meilisearch_http::data::Data;
use meilisearch_http::option::Opt;
use meilisearch_http::routes;
use meilisearch_http::routes::index::index_update_callback;

mod analytics;

#[cfg(target_os = "linux")]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

pub fn main() -> Result<(), MainError> {

    let opt = Opt::from_args();

    match opt.env.as_ref() {
        "production" => {
            if opt.master_key.is_none() {
                return Err("In production mode, the environment variable MEILI_MASTER_KEY is mandatory".into());
            }
            env_logger::init();
        },
        "development" => {
            env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();
        },
        _ => unreachable!(),
    }

    if !opt.no_analytics {
        thread::spawn(analytics::analytics_sender);
    }

    let data = Data::new(opt.clone());

    let data_cloned = data.clone();
    data.db.set_update_callback(Box::new(move |name, status| {
        index_update_callback(name, &data_cloned, status);
    }));

    print_launch_resume(&opt, &data);

    let mut app = tide::with_state(data);

    app.middleware(Cors::new());
    app.middleware(RequestLogger::new());

    routes::load_routes(&mut app);

    task::block_on(app.listen(opt.http_addr))?;
    Ok(())
}


pub fn print_launch_resume(opt: &Opt, data: &Data) {
    let ascii_name = r#"
888b     d888          d8b 888 d8b  .d8888b.                                    888
8888b   d8888          Y8P 888 Y8P d88P  Y88b                                   888
88888b.d88888              888     Y88b.                                        888
888Y88888P888  .d88b.  888 888 888  "Y888b.    .d88b.   8888b.  888d888 .d8888b 88888b.
888 Y888P 888 d8P  Y8b 888 888 888     "Y88b. d8P  Y8b     "88b 888P"  d88P"    888 "88b
888  Y8P  888 88888888 888 888 888       "888 88888888 .d888888 888    888      888  888
888   "   888 Y8b.     888 888 888 Y88b  d88P Y8b.     888  888 888    Y88b.    888  888
888       888  "Y8888  888 888 888  "Y8888P"   "Y8888  "Y888888 888     "Y8888P 888  888
"#;

    println!("{}", ascii_name);

    info!("Database path: {:?}", opt.db_path);
    info!("Start server on: {:?}", opt.http_addr);
    info!("Environment: {:?}", opt.env);
    info!("Commit SHA: {:?}", env!("VERGEN_SHA").to_string());
    info!("Build date: {:?}", env!("VERGEN_BUILD_TIMESTAMP").to_string());
    info!("Package version: {:?}", env!("CARGO_PKG_VERSION").to_string());

    if let Some(master_key) = &data.api_keys.master {
        info!("Master Key: {:?}", master_key);

        if let Some(private_key) = &data.api_keys.private {
            info!("Private Key: {:?}", private_key);
        }

        if let Some(public_key) = &data.api_keys.public {
            info!("Public Key: {:?}", public_key);
        }
    } else {
        info!("No master key found; The server will have no securities.\
            If you need some protection in development mode, please export a key. export MEILI_MASTER_KEY=xxx");
    }

    info!("If you need extra information; Please refer to the documentation: http://docs.meilisearch.com");
    info!("If you want to support us or help us; Please consult our Github repo: http://github.com/meilisearch/meilisearch");
    info!("If you want to contact us; Please chat with us on http://meilisearch.com or by email to bonjour@meilisearch.com");
}
