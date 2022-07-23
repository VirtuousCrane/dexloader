use std::str::FromStr;
use std::env;

use dexloader::manga::Manga;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Dealing with command line arguments
    let args: Vec<String> = env::args().collect();
    let url = &args[1];

    let mut argument_iterator = args.iter();
    argument_iterator.next();
    argument_iterator.next();

    let mut output_path: &str = ".epub";
    let mut limit: Option<i32> = Some(6);
    let mut start = 0;
    let mut end: Option<i32> = None;
    let mut single = false;
    
    while let Some(val) = argument_iterator.next() {
        if val == "-o" || val == "--output" {
            match argument_iterator.next() {
                Some(path) => output_path = path.trim(),
                None => panic!("No output path specified")
            };
        } else if val == "-l" || val == "--limit" {
            match argument_iterator.next() {
                Some(l) => {
                    let lim = i32::from_str(l)
                        .expect("Failed to parse limit value");
                    limit = Some(lim);
                },
                None => panic!("No limit value specified")
            }
        } else if val == "-s" || val == "--start" {
            match argument_iterator.next() {
                Some(s) => {
                    let st = i32::from_str(s)
                        .expect("Failed to parse start value");
                    start = st;
                },
                None => panic!("No start value specified")
            }
        } else if val == "-e" || val == "--end" {
            match argument_iterator.next() {
                Some(e) => {
                    let ed = i32::from_str(e)
                        .expect("Failed to parse end value");
                    end = Some(ed);
                },
                None => panic!("No end value specified")
            }
        } else if val == "--single" {
            single = true;
        }
    }

    // Driving the program
    //let mut url = String::new();

    //io::stdin()
    //    .read_line(&mut url)
    //    .expect("Failed to read line");
    let url = url.trim();
    
    let mut manga = Manga::from(url);
    let mut total = 0;
    loop {
        if total != 0 && start > total {
            if single {
                manga.generate_epub(output_path).await;
            }
            break;
        }

        match end {
            Some(i) => {
                if i <= start {
                    break;
                }
            },
            None => (),
        }
        
        manga.get_chapters(limit, start, end).await;
        total = manga.get_total().unwrap();

        manga.download_chapters(!single).await
            .expect("Failed to download chapters");
        
        if !single {
            let temp = format!("File_{}_", start/limit.unwrap());
            let path = temp + output_path;
            println!("Saving {}", path);
            manga.generate_epub(&path).await;
        } else {
            println!("Saving {}", output_path);
            manga.generate_epub(&output_path).await;
            break;
        }

        start += limit.unwrap();
    }

    Ok(())
}
