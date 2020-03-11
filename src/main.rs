use std::env;

use tokio::runtime::Runtime;

pub mod constants;
pub mod operations;
pub mod util;

use crate::operations::{get_all_shows, get_show, get_slice_shows, list_shows};

/// Entrypoint for syf
/// 
/// Quick and dirty match expression for command line args
fn main() {
    let args: Vec<String> = env::args().collect();
    let mut runtime = Runtime::new().unwrap();

    if let Some(arg) = args.get(1) {
        match &arg[..] {
            "--help" => {
                let message = r#"
                syf (steal your face) commands:

                list                        lists all available Grateful Dead shows from archive.org by name
                fetch {name}                downloads show matching the specified name to current working directory
                fetch slice {start} {end}   downloads all shows between the specified start and end show (inclusive)
                fetch all                   downloads all available shows to the current working directory
                "#;
                println!("{}", message);
            },
            "list" => {
                match Runtime::block_on(&mut runtime, list_shows()) {
                    Ok(_) => println!("Successfully listed all shows."),
                    Err(e) => println!("Failed to list all shows: {}", e),
                };
            },
            "fetch" => {
                if let Some(target) = args.get(2) {
                    if target == "all" {
                        match Runtime::block_on(&mut runtime, get_all_shows()) {
                            Ok(_) => println!("Successfully downloaded all shows."),
                            Err(e) => println!("Failed to download all shows: {}", e),
                        };
                    } else if target == "slice" {
                        if let Some(start) = args.get(3) {
                            if let Some(end) = args.get(4) {
                                match Runtime::block_on(&mut runtime, get_slice_shows(start, end)) {
                                    Ok(_) => println!("Successfully downloaded show slice."),
                                    Err(e) => println!("Failed to download show slice: {}", e),
                                }
                            } else {
                                println!("Please specify an end show for the slice.");
                            }
                        } else {
                            println!("Please specify a start show for the slice.");
                        }
                    } else {
                        match Runtime::block_on(&mut runtime, get_show(target)) {
                            Ok(_) => println!("Successfully downloaded show: \"{}\"", target),
                            Err(e) => println!("Failed to download show: \"{}\", {}", target, e),
                        }
                    }
                } else {
                    println!(
                        "Please specify a show to fetch, `all`, or `slice {} {}`",
                        "{first_show}", "{last_show}"
                    );
                }
            },
            _ => println!(
                "Invalid command. Valid options are `list`, `fetch $showname`, or `fetch all`"
            ),
        };
    } else {
        let message = "You need to specify something to do. \
        Valid options are `list`, `fetch $showname`, or `fetch all`";
        println!("{}", message);
    }
}
