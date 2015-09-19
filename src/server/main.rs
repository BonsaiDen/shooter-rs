#[macro_use]
extern crate clap;
extern crate shared;

fn main() {

    let args = clap::App::new("server")
        .version(&crate_version!())
        .author("Ivo Wetzel <ivo.wetzel@googlemail.com>")
        .about("Server")
        .arg(clap::Arg::with_name("address:port")
            .help("Local server address to bind to.")
            .index(1)
        ).get_matches();

}

