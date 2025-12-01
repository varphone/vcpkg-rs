extern crate vcpkg;

use clap::{Arg, Command};
use std::env;

fn main() {
    let app = Command::new("vcpkg library finder")
        .about("Allows examining what vcpkg will find in a build script")
        .subcommand_required(true)
        .arg(
            Arg::new("target")
                .short('t')
                .long("target")
                .value_name("RUST TARGET TRIPLE")
                .help("the rust toolchain triple to find libraries for")
                .default_value("x86_64-pc-windows-msvc"),
        )
        .subcommand(
            Command::new("probe")
                .about("try to find a package")
                .arg(
                    Arg::new("package")
                        .index(1)
                        .required(true)
                        .help("probe for a library and display paths and cargo metadata"),
                )
                .arg(
                    Arg::new("linkage")
                        .short('l')
                        .long("linkage")
                        .value_parser(["dll", "static"]),
                ),
        );

    let matches = app.get_matches();

    // set TARGET as if we are running under cargo
    unsafe {
        env::set_var("TARGET", matches.get_one::<String>("target").unwrap());
    }

    if let Some(matches) = matches.subcommand_matches("probe") {
        let lib_name = matches.get_one::<String>("package").unwrap().as_str();

        let mut cfg = vcpkg::Config::new();
        cfg.cargo_metadata(false);
        cfg.copy_dlls(false);
        if let Some(linkage) = matches.get_one::<String>("linkage") {
            match linkage.as_str() {
                "dll" => {
                    remove_vars();
                    unsafe {
                        env::set_var("VCPKGRS_DYNAMIC", "1");
                    }
                }
                "static" => {
                    remove_vars();
                    unsafe {
                        env::set_var("CARGO_CFG_TARGET_FEATURE", "crt-static");
                    }
                }
                _ => unreachable!(),
            }
        }

        match cfg.find_package(lib_name) {
            Ok(lib) => {
                println!("Found library {}", lib_name);

                if !lib.include_paths.is_empty() {
                    println!("Include paths:");
                    for line in &lib.include_paths {
                        println!("  {}", line.as_os_str().to_str().unwrap());
                    }
                }

                if !lib.link_paths.is_empty() {
                    println!("Library paths:");
                    for line in &lib.link_paths {
                        println!("  {}", line.as_os_str().to_str().unwrap());
                    }
                }

                if !lib.link_paths.is_empty() {
                    println!("Runtime Library paths:");
                    for line in &lib.dll_paths {
                        println!("  {}", line.as_os_str().to_str().unwrap());
                    }
                }

                if !lib.cargo_metadata.is_empty() {
                    println!("Cargo metadata:");
                    for line in &lib.cargo_metadata {
                        println!("  {}", line);
                    }
                }
                if !lib.found_dlls.is_empty() {
                    println!("Found DLLs:");
                    for line in &lib.found_dlls {
                        println!("  {}", line.display());
                    }
                }
                if !lib.found_libs.is_empty() {
                    println!("Found libs:");
                    for line in &lib.found_libs {
                        println!("  {}", line.display());
                    }
                }
                if !lib.found_names.is_empty() {
                    println!("Libraries linking names:");
                    for line in &lib.found_names {
                        println!("  {}", line);
                    }
                }
            }
            Err(err) => {
                println!("Failed:  {}", err);
            }
        }
    }
}

fn remove_vars() {
    unsafe {
        env::remove_var("VCPKGRS_DYNAMIC");
        env::remove_var("CARGO_CFG_TARGET_FEATURE");
    }
}
