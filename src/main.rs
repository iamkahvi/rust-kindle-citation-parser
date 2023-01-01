use regex::Regex;
use std::fs;

fn main() {
    println!("Hello, world!");
    let input_file = std::env::args().nth(1).expect("no input file given");
    let output_file = std::env::args().nth(2).expect("no output file given");

    // if input == "--help" || first_arg == "-h" {
    //     println!("nr - Prepend line numbers to STDIN");
    //     println!();
    //     println!("Usage:");
    //     println!("    <your command> | nr [minwidth] [offset]");
    //     println!();
    //     println!("Options:");
    //     println!("    [minwidth]    The minimum width of number column");
    //     println!("                  Select 0 for no minimum width");
    //     println!();
    //     println!("    [offset]      Starting line index. Default is 1");
    //     return;
    // }

    println!("input_file: {}", input_file);
    println!("output_file: {}", output_file);

    let text = fs::read_to_string(&input_file).unwrap();
    let items = text.split("==========");

    let highlights: Vec<Highlight> = items
        .flat_map(|x| {
            let res = process_item(x.to_string());
            match res {
                None => return vec![],
                Some(h) => return vec![h],
            }
        })
        .collect();

    print!("{:?}", highlights.get(1));

    // print!("input: {}", temp);
}

#[derive(Debug)]
struct Highlight {
    book: String,
    author: String,
    quote: String,
    page: String,
    location: String,
    date_added: String,
}

fn process_item(item: String) -> Option<Highlight> {
    let lines = item
        .lines()
        .map(|x| x.trim_start_matches("\u{feff}"))
        .filter(|x| !x.is_empty())
        .collect::<Vec<&str>>();

    println!("lines: {:?}", lines);
    if lines.len() < 3 {
        eprintln!("Invalid item: {}", item);
        return None;
    }

    let (book, author) = match parse_first_line(lines[0].to_string()) {
        Some((book, author)) => (book, author),
        None => return None,
    };

    let (page, location, date_added) = match parse_second_line(lines[1].to_string()) {
        Some((page, location, date_added)) => (page, location, date_added),
        None => return None,
    };

    let h = Highlight {
        book,
        author,
        quote: String::new(),
        page,
        location,
        date_added,
    };

    return Some(h);
}

// returns book title and author string tuple
fn parse_first_line(line: String) -> Option<(String, String)> {
    let mut author_name = String::new();

    let re = Regex::new(r"^(.*) \((.*), (.*)\)$").unwrap();

    print!("line: {}", line);

    if !re.is_match(&line) {
        println!("Invalid first line: {}", line);
        return None;
    }
    let caps = re.captures(&line).unwrap();

    let book = caps.get(1).unwrap().as_str().to_owned();

    let author_last_name = caps.get(2).unwrap().as_str();
    let author_first_name = caps.get(3).unwrap().as_str();

    author_name.push_str(author_first_name);
    author_name.push_str(" ");
    author_name.push_str(author_last_name);

    return Some((book, author_name));
}

// returns page, location, date added
fn parse_second_line(line: String) -> Option<(String, String, String)> {
    let mut page = String::new();
    let mut location = String::new();
    let mut date_added = String::new();

    let re1 = Regex::new(r"^- [^|]*page (.*) \| Location (.*) \| Added on (.*)$").unwrap();
    let re2 = Regex::new(r"^- [^|]*Location (.*) \| Added on (.*)$").unwrap();

    if re1.is_match(&line) {
        // - Your Highlight on page 293 | Location 4131-4131 | Added on Monday, December 19, 2022 12:50:19 PM
        let caps = re1.captures(&line).unwrap();

        page = caps.get(0).unwrap().as_str().to_owned();
        location = caps.get(1).unwrap().as_str().to_owned();
        date_added = caps.get(2).unwrap().as_str().to_owned();
    } else if re2.is_match(&line) {
        // - Your Highlight on Location 138-140 | Added on Monday, December 19, 2022 10:29:07 PM
        let caps = re2.captures(&line).unwrap();

        location = caps.get(1).unwrap().as_str().to_owned();
        date_added = caps.get(2).unwrap().as_str().to_owned();
    } else {
        eprintln!("Invalid second line: {}", line);
        return None;
    }

    return Some((page, location, date_added));
}
