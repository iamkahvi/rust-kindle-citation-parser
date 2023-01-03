use chrono::{TimeZone, Utc};
use regex::Regex;
use serde::Serialize;
use serde_json;
use std::fs;

#[derive(Debug, Serialize)]

struct Location {
    start: i32,
    end: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Highlight {
    book: String,
    author: String,
    quote: String,
    page: Option<i32>,
    location: Location,
    date_added: i64,
}

fn main() {
    let input_file = std::env::args().nth(1).expect("no input file given");
    let output_file = std::env::args().nth(2).expect("no output file given");
    let book_regex = std::env::args().nth(3);

    println!("input_file: {}", input_file);
    println!("output_file: {}", output_file);
    match &book_regex {
        Some(regex) => println!("book_regex: {}", regex),
        _ => (),
    }

    let text = fs::read_to_string(&input_file).unwrap();
    let items = text.split("==========");

    let highlights: Vec<Highlight> = items
        .filter_map(|x| process_item(x.to_string()))
        .filter(|h| {
            return match &book_regex {
                Some(regex) => {
                    let re = Regex::new(regex).unwrap();
                    re.is_match(&h.book)
                }
                None => true,
            };
        })
        .collect();

    let serialized = serde_json::to_string(&highlights).unwrap();

    fs::write(output_file, serialized).expect("Unable to write file");
}

fn process_item(item: String) -> Option<Highlight> {
    let lines = item
        .lines()
        .map(|x| x.trim_start_matches("\u{feff}"))
        .filter(|x| !x.is_empty())
        .collect::<Vec<&str>>();

    if lines.len() < 3 {
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

    let quote = lines[2].to_string();

    let h = Highlight {
        book,
        author,
        quote,
        page,
        location,
        date_added,
    };

    return Some(h);
}

// returns book title and author string tuple
fn parse_first_line(line: String) -> Option<(String, String)> {
    let mut book = String::new();
    let mut author_name = String::new();

    let re1 = Regex::new(r"^(.*) \(([^,]*), (.*)\)$").unwrap();
    let re2 = Regex::new(r"^(.*) \(([^,]*) (.*)\)$").unwrap();
    let re3 = Regex::new(r"^(.*) \((.*)\)$").unwrap();

    if re1.is_match(&line) {
        let caps = re1.captures(&line).unwrap();
        book = caps.get(1).unwrap().as_str().to_owned();
        caps.expand(r"$3 $2", &mut author_name);
    } else if re2.is_match(&line) {
        let caps = re2.captures(&line).unwrap();
        book = caps.get(1).unwrap().as_str().to_owned();
        caps.expand(r"$2 $3", &mut author_name);
    } else if re3.is_match(&line) {
        let caps = re3.captures(&line).unwrap();
        book = caps.get(1).unwrap().as_str().to_owned();
        author_name = caps.get(2).unwrap().as_str().to_owned();
    } else if !line.is_empty() {
        book = line;
        author_name = "Unknown".to_string();
    } else {
        println!("Invalid first line: {}", line);
        return None;
    }

    return Some((book, author_name));
}

// returns page, location, date added
fn parse_second_line(line: String) -> Option<(Option<i32>, Location, i64)> {
    let mut page = None;
    let mut location = Location { start: 0, end: 0 };
    let mut date_added = 0;

    let re1 =
        Regex::new(r"^- Your [Hh]ighlight[^|]*page (.*) \| [Ll]ocation (.*) \| Added on (.*)$")
            .unwrap();
    let re2 = Regex::new(r"^- Your [Hh]ighlight[^|]*[Ll]ocation (.*) \| Added on (.*)$").unwrap();

    if re1.is_match(&line) {
        // - Your Highlight on page 293 | Location 4131-4131 | Added on Monday, December 19, 2022 12:50:19 PM
        let caps = re1.captures(&line).unwrap();

        page = Some(caps.get(1).unwrap().as_str().parse::<i32>().unwrap());

        let location_string = caps.get(2).unwrap().as_str().to_owned();
        if let Some(l) = parse_location(location_string) {
            location = l;
        }

        date_added = parse_datetime(caps.get(3).unwrap().as_str().to_owned());
    } else if re2.is_match(&line) {
        // - Your Highlight on Location 138-140 | Added on Monday, December 19, 2022 10:29:07 PM
        let caps = re2.captures(&line).unwrap();

        let location_string = caps.get(1).unwrap().as_str().to_owned();
        if let Some(l) = parse_location(location_string) {
            location = l;
        }

        date_added = parse_datetime(caps.get(2).unwrap().as_str().to_owned());
    } else {
        println!("Invalid second line: {}", line);
        return None;
    }

    return Some((page, location, date_added));
}

fn parse_datetime(datetime_string: String) -> i64 {
    let fmt1 = "%A, %B %d, %Y %l:%M:%S %p";
    let fmt2 = "%A, %d %B %Y %H:%M:%S";

    let datetime_res1: Result<chrono::DateTime<Utc>, chrono::ParseError> =
        TimeZone::datetime_from_str(&Utc, &datetime_string, fmt1);
    let datetime_res2: Result<chrono::DateTime<Utc>, chrono::ParseError> =
        TimeZone::datetime_from_str(&Utc, &datetime_string, fmt2);

    match (datetime_res1, datetime_res2) {
        (Ok(datetime), _) => return datetime.timestamp(),
        (_, Ok(datetime)) => return datetime.timestamp(),
        _ => {
            println!("Invalid datetime: {}", datetime_string);
            return 0;
        }
    }
}

fn parse_location(location_string: String) -> Option<Location> {
    let re = Regex::new(r"^(.*)-(.*)$").unwrap();

    if re.is_match(location_string.as_str()) {
        let caps = re.captures(location_string.as_str()).unwrap();

        let start = caps.get(1).unwrap().as_str().parse::<i32>().unwrap();
        let end = caps.get(2).unwrap().as_str().parse::<i32>().unwrap();

        return Some(Location { start, end });
    } else {
        eprintln!("Invalid location: {}", location_string);
        return None;
    }
}
