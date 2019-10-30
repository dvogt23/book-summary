use serde_json::Value as jsonValue;
use std::env;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;
use toml::Value;
use walkdir::{DirEntry, WalkDir};

mod book;
use book::Chapter;
use book::Format;

#[derive(Debug, PartialEq)]
enum SummaryError {}

impl fmt::Display for SummaryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self)
    }
}

type Result<T> = std::result::Result<T, Box<SummaryError>>;

#[derive(StructOpt, Debug)]
#[structopt()]
struct Opt {
    /// Activate debug mode
    #[structopt(name = "debug", short, long)]
    debug: bool,

    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: u8,

    /// Title from md file header?
    #[structopt(name = "mdheader", short, long)]
    mdheader: bool,

    /// Format md/git book
    #[structopt(name = "format", short, long, default_value = "md")]
    format: Format,

    /// Title for summary
    #[structopt(name = "title", short, long, default_value = "Summary")]
    title: String,

    /// Start with following chapters (space seperate)
    #[structopt(name = "sort", short, long)]
    sort: Option<Vec<String>>,

    /// Output file
    #[structopt(name = "outputfile", short, long, default_value = "SUMMARY.md")]
    outputfile: String,

    /// Notes dir where to parse all your notes from
    #[structopt(name = "notesdir", short, long, default_value = ".")]
    dir: PathBuf,

    /// Overwrite existing SUMMARY.md file
    #[structopt(name = "yes", short, long = "overwrite")]
    yes: bool,
}

fn main() {
    let mut opt = Opt::from_args();

    // print opt in verbose level 3
    if opt.verbose > 2 {
        println!("{:?}", opt);
        println!("{:?}", env::current_dir().unwrap().display());
    }

    // parse book.js OR book.toml
    match opt.format {
        Format::Md(_) => parse_config_file(&format!("{}{}", opt.dir.display(), "/book.toml"), &mut opt),
        Format::Git(_) => {
            parse_config_file(&format!("{}{}", opt.dir.display(), "/book.json"), &mut opt);
            parse_config_file(&format!("{}{}", opt.dir.display(), "/book.js"), &mut opt);
        },
    }

    if opt.dir == PathBuf::from("./") {
        opt.dir = env::current_dir().unwrap();
    }

    if !opt.dir.is_dir() {
        eprintln!("Error: Path {} not found!", opt.dir.display());
        std::process::exit(1)
    }

    let entries = match get_dir(&opt.dir, &opt.outputfile) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("Error: {:?}", err);
            std::process::exit(1)
        }
    };

    // SUMMARY.md file check if exists
    if Path::new(&format!("{}/{}", &opt.dir.display(), &opt.outputfile)).exists() && !opt.yes {
        loop {
            println!(
                "File {} already exists, do you want to overwrite it? [Y/n]",
                &opt.outputfile
            );
            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) if &input == "y\n" || &input == "Y\n" || &input == "\n" => break,
                Ok(_) if &input == "n\n" || &input == "N\n" => return,
                _ => {}
            }
        }
    }

    if opt.verbose > 2 {
        dbg!(&entries);
    }

    let book = Chapter::new(opt.title, &entries);

    create_file(
        &opt.dir.to_str().unwrap(),
        &opt.outputfile,
        // &book.get_summary_file(&opt.format),
        &book.get_summary_file(&opt.format, &opt.sort),
    );

    if opt.verbose > 2 {
        dbg!(&book);
    }
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn get_dir(dir: &PathBuf, outputfile: &str) -> Result<Vec<String>> {
    let mut entries: Vec<String> = vec![];
    for direntry in WalkDir::new(dir)
        .sort_by(|a, b| a.file_name().cmp(b.file_name()))
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok())
    {
        // entry without:
        // - given root folder
        // - plain dirnames
        // - not md files
        // - not SUMMARY.md file
        let entry = direntry
            .path()
            .to_str()
            .unwrap()
            .chars()
            .skip(dir.to_str().unwrap().len() + 1)
            .collect::<String>();
        if !entry.is_empty()
            && !entry.eq(outputfile)
            && !entry.to_lowercase().eq("readme.md")
            && entry.contains(".md")
        {
            entries.push(entry);
        }
    }
    Ok(entries)
}

fn parse_config_file(path: &str, opt: &mut Opt) {
    let path = Path::new(path);

    if !path.exists() {
        if opt.verbose > 2 {
            eprintln!("Book config file {} not found.", path.display());
        }
        return;
    }

    let mut file = match File::open(&path) {
        Err(why) => panic!(
            "Error: Couldn't open {}: {}",
            path.display(),
            why.description()
        ),
        Ok(file) => file,
    };

    let mut content = String::new();
    match file.read_to_string(&mut content) {
        Err(why) => panic!(
            "Error: Couldn't read {}: {}",
            path.display(),
            why.description()
        ),
        Ok(_) => {}
    }

    if opt.verbose > 2 {
        println!("Found book config file: {}", path.display());
    }

    let ext: &str = path.extension().unwrap().to_str().unwrap();

    match ext {
        "toml" => {
            let values = content.parse::<Value>().unwrap();
            if opt.dir.to_str().eq(&Some(".")) {
                if let Some(src) = values["book"]["src"].as_str() {
                    if opt.verbose > 2 {
                        println!("Found `src` in book.toml: {}", src);
                    }
                    opt.dir = PathBuf::from(src);
                }
            }

            if opt.title.eq("Summary") {
                if let Some(title) = values["book"]["title"].as_str() {
                    if opt.verbose > 2 {
                        println!("Found `title` in book.toml: {}", title);
                    }
                    opt.title = title.to_string();
                }
            }
        }
        "js" | "json" => {
            let values: jsonValue = serde_json::from_str(&content).unwrap();
            if opt.dir.to_str().eq(&Some(".")) {
                if let Some(src) = values["root"].as_str() {
                    if opt.verbose > 2 {
                        println!("Found `root` in book.{}: {}", ext, src);
                    }
                    opt.dir = PathBuf::from(src);
                }
            }

            if opt.title.eq("Summary") {
                if let Some(title) = values["title"].as_str() {
                    if opt.verbose > 2 {
                        println!("Found `title` in book.{}: {}", ext, title);
                    }
                    opt.title = title.to_string();
                }
            }
        }
        _ => {}
    }
}

fn create_file(path: &str, filename: &str, content: &str) {
    let filepath = format!("{}/{}", path, filename);
    let path = Path::new(&filepath);
    let display = path.display();

    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => panic!("Couldn't create {}: {}", display, why.description()),
        Ok(file) => file,
    };

    // Write the `LOREM_IPSUM` string to `file`, returns `io::Result<()>`
    match file.write_all(content.as_bytes()) {
        Err(why) => panic!("Couldn't write to {}: {}", display, why.description()),
        Ok(_) => println!("Successfully create {}", display),
    }
}

/* ------------------------- TEST --------------------------------- */
#[cfg(test)]
mod tests {
    use super::*;

    const TITLE: &str = "Summary";
    const FORMAT: Format = Format::Git('*');

    // # get file list: no hidden files, filepaths from given folder as root
    #[test]
    fn get_file_list_test() {
        let expected = Ok(vec![
            "about.md".to_string(),
            "chapter1/FILE.md".to_string(),
            "chapter1/file1.md".to_string(),
            "chapter2/FILE1.md".to_string(),
            "chapter2/README.md".to_string(),
            "chapter2/file2.md".to_string(),
            "chapter2/subchap/info.md".to_string(),
            "chapter3/file1.md".to_string(),
            "chapter3/file2.md".to_string(),
            "chapter3/file3.md".to_string(),
        ]);
        assert_eq!(
            expected,
            get_dir(&PathBuf::from(r"./examples/gitbook/book"), &"SUMMARY.md")
        );
    }

    #[test]
    fn create_struct_empty_test() {
        // # empty list

        let input: Vec<String> = vec![];
        let expected: Chapter = Chapter {
            name: TITLE.to_string(),
            files: vec![],
            chapter: vec![],
        };

        let book = Chapter::new(TITLE.to_string(), &input);

        assert_eq!(expected, book);
    }

    #[test]
    fn create_struct_onefile_test() {
        // # only one file
        let input: Vec<String> = vec!["file.md".to_string()];
        let expected: Chapter = Chapter {
            name: TITLE.to_string(),
            files: vec!["file.md".to_string()],
            chapter: vec![],
        };

        let book = Chapter::new(TITLE.to_string(), &input);

        assert_eq!(expected, book);
    }

    #[test]
    fn create_struct_onechapter_test() {
        // # only one chapter
        let input: Vec<String> = vec!["chapter1/file1.md".to_string()];

        let expected: Chapter = Chapter {
            name: TITLE.to_string(),
            files: vec![],
            chapter: vec![Chapter {
                name: "chapter1".to_string(),
                files: vec!["chapter1/file1.md".to_string()],
                chapter: vec![],
            }],
        };

        let book = Chapter::new(TITLE.to_string(), &input);

        assert_eq!(expected, book);
    }

    #[test]
    fn create_struct_subchapter_test() {
        // # chapter with subchapters
        let input: Vec<String> = vec![
            "chapter1/file1.md".to_string(),
            "chapter1/subchap/file1.md".to_string(),
        ];

        let expected: Chapter = Chapter {
            name: TITLE.to_string(),
            files: vec![],
            chapter: vec![Chapter {
                name: "chapter1".to_string(),
                files: vec!["chapter1/file1.md".to_string()],
                chapter: vec![Chapter {
                    name: "subchap".to_string(),
                    files: vec!["chapter1/subchap/file1.md".to_string()],
                    chapter: vec![],
                }],
            }],
        };

        let book = Chapter::new(TITLE.to_string(), &input);

        assert_eq!(expected, book);
    }

    // 2. Markdown output for entry in chapter
    //      - format (md/git)
    //      - titlecase for entry
    //      - remove pre numbers in entry
    #[test]
    fn md_output_onefile_test() {
        let list_char: char = match FORMAT {
            Format::Md(c) => c,
            Format::Git(c) => c,
        };

        // only one file
        let input: Vec<String> = vec!["file1.md".to_string()];

        let expected: &str = &format!("# {}\n\n{} [File1](file1.md)\n", TITLE, list_char);

        let book = Chapter::new(TITLE.to_string(), &input);
        dbg!(&book);

        assert_eq!(expected, book.get_summary_file(&FORMAT, &None));
    }

    #[test]
    fn md_output_onechapter_test() {
        let list_char: char = match FORMAT {
            Format::Md(c) => c,
            Format::Git(c) => c,
        };

        // only one file
        let input: Vec<String> = vec!["file1.md".to_string(), "chapter1/file1.md".to_string()];

        let expected: &str = &format!(
            "# {0}\n\n{1} [File1](file1.md)\n{1} Chapter1\n    {1} [File1](chapter1/file1.md)\n",
            TITLE, list_char
        );

        let book = Chapter::new(TITLE.to_string(), &input);

        assert_eq!(expected, book.get_summary_file(&FORMAT, &None));
    }

    #[test]
    fn md_output_subchapter_test() {
        let list_char: char = match FORMAT {
            Format::Md(c) => c,
            Format::Git(c) => c,
        };

        // only one file
        let input: Vec<String> = vec![
            "chapter1/file1.md".to_string(),
            "chapter1/subchap/file1.md".to_string(),
        ];

        let expected: &str = &format!(
            "# {0}\n\n{1} Chapter1\n    {1} [File1](chapter1/file1.md)\n    {1} Subchap\n        {1} [File1](chapter1/subchap/file1.md)\n",
            TITLE, list_char
        );

        let book = Chapter::new(TITLE.to_string(), &input);

        assert_eq!(expected, book.get_summary_file(&FORMAT, &None));
    }

    #[test]
    fn md_simple_structure_test() {
        let input = vec![
            "part1/README.md".to_string(),
            "part1/WritingIsGood.md".to_string(),
            "part1/GitbookIsNice.md".to_string(),
            "part2/README.md".to_string(),
            "part2/First_part_of_part_2.md".to_string(),
            "part2/Second_part_of_part_2.md".to_string(),
        ];

        let expected = r#"# Summary

* [Part1](part1/README.md)
    * [WritingIsGood](part1/WritingIsGood.md)
    * [GitbookIsNice](part1/GitbookIsNice.md)
* [Part2](part2/README.md)
    * [First Part of Part 2](part2/First_part_of_part_2.md)
    * [Second Part of Part 2](part2/Second_part_of_part_2.md)
"#;

        let book = Chapter::new(TITLE.to_string(), &input);

        assert_eq!(expected, book.get_summary_file(&FORMAT, &None));
    }

    #[test]
    fn parse_config_test() {
        let bookjson = "./examples/gitbook/book.json";
        let booktoml = "./examples/mdbook/book.toml";

        // opt with default values
        let mut opt = Opt {
            debug: false,
            verbose: 3,
            mdheader: false,
            format: FORMAT,
            title: "Summary".to_string(),
            sort: None,
            outputfile: "SUMMARY.md".to_string(),
            dir: PathBuf::from("."),
            yes: true,
        };

        parse_config_file(booktoml, &mut opt);

        assert_eq!("src", format!("{}", opt.dir.display()));
        assert_eq!("MyMDBook", opt.title);

        opt.dir = PathBuf::from(".");
        opt.title = "Summary".to_string();

        parse_config_file(bookjson, &mut opt);

        assert_eq!("book", format!("{}", opt.dir.display()));
        assert_eq!("My title", opt.title);
    }

    #[test]
    fn sort_chapter_test() {
        let input = vec![
            "part1/README.md".to_string(),
            "part1/WritingIsGood.md".to_string(),
            "part2/GitbookIsNice.md".to_string(),
            "part2/README.md".to_string(),
            "part3/file.md".to_string(),
            "part4/file.md".to_string(),
        ];

        let expected = r#"# Summary

* Part4
    * [File](part4/file.md)
* Part3
    * [File](part3/file.md)
* [Part1](part1/README.md)
    * [WritingIsGood](part1/WritingIsGood.md)
* [Part2](part2/README.md)
    * [GitbookIsNice](part2/GitbookIsNice.md)
"#;

        let book = Chapter::new(TITLE.to_string(), &input);

        assert_eq!(
            expected,
            book.get_summary_file(
                &FORMAT,
                &Some(vec![
                    "PART4".to_string(),
                    "part5".to_string(),
                    "part3".to_string()
                ])
            )
        );
    }
}
