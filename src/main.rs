use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Some(arg) = args.get(1) {
        match &arg[..] {
            "list" => println!("Fetching list of shows.."),
            "fetch" => {
                if let Some(target) = args.get(2) {
                    println!("Fetching show {}", target)
                } else {
                    println!("Please specify a show to fetch or `all`")
                }
            }
            _ => println!(
                "Invalid command. Valid options are `list`, `fetch $showname`, or `fetch all`"
            ),
        }
    } else {
        let message = "You need to specify something to do. \
        Valid options are `list`, `fetch $showname`, or `fetch all`";
        println!("{}", message);
    }
}
