use std::env;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;
use walkdir::{DirEntry, WalkDir};

mod book;
use book::Chapter;

#[derive(Debug, PartialEq)]
enum SummaryError {
    UnknownPath,
    TooSmall,
}

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
    format: String,

    /// Title for summary
    #[structopt(name = "title", short, long, default_value = "Summary")]
    title: String,

    /// Default sort by asc, but first start with following chapters
    #[structopt(name = "sort", short, long)]
    sort: Option<Vec<String>>,

    /// Output file
    #[structopt(name = "outputfile", short, long, default_value = "SUMMARY.md")]
    outputfile: String,

    /// Notes dir where to parse all your notes from
    #[structopt(name = "notesdir", short, long, default_value = "./")]
    dir: PathBuf,
}

fn main() {
    let mut opt = Opt::from_args();

    // print opt in verbose level 3
    if opt.verbose > 2 {
        println!("{:?}", opt);
        println!("{:?}", env::current_dir().unwrap().display());
    }

    if opt.dir == PathBuf::from("./") {
        opt.dir = env::current_dir().unwrap();
    }

    if !opt.dir.is_dir() {
        println!("Error: Path {} not found!", opt.dir.display());
    }

    let entries = match get_dir(&opt.dir, &opt.outputfile) {
        Ok(e) => e,
        Err(err) => panic!(err),
    };

    //    let book = create_chapter(&entries, opt.title);

    let book = Chapter::new(opt.title, &entries);

    create_file(
        &opt.dir.to_str().unwrap(),
        &opt.outputfile,
        &book.get_summary_file(&opt.format),
    );

    dbg!(&book);

    // sort_filelist(&mut entries, opt.sort.as_ref());
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
            && (entry.contains("/") || entry.contains(".md"))
        {
            entries.push(entry);
        }
    }
    Ok(entries)
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
    const LIST_CHAR: char = '-';

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
            "chapter2/subchap".to_string(),
            "chapter2/subchap/info.md".to_string(),
            "chapter3/file1.md".to_string(),
            "chapter3/file2.md".to_string(),
            "chapter3/file3.md".to_string(),
        ]);
        assert_eq!(
            expected,
            get_dir(&PathBuf::from(r"./examples/book"), &"SUMMARY.md")
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
        // only one file
        let input: Vec<String> = vec!["file1.md".to_string()];

        let expected: &str = &format!("# {}\n\n{} [File1](./file1.md)\n", TITLE, LIST_CHAR);

        let book = Chapter::new(TITLE.to_string(), &input);

        assert_eq!(expected, book.get_summary_file("md"));
    }

    #[test]
    fn md_output_onechapter_test() {
        // only one file
        let input: Vec<String> = vec!["file1.md".to_string(), "chapter1/file1.md".to_string()];

        let expected: &str = &format!(
            "# {}\n\n{} [File1](./file1.md)\n- [Chapter1]()\n\t- [File1](./chapter1/file1.md)",
            TITLE, LIST_CHAR
        );

        let book = Chapter::new(TITLE.to_string(), &input);

        assert_eq!(expected, book.get_summary_file("md"));
    }

    #[test]
    fn md_output_subchapter_test() {
        // only one file
        let input: Vec<String> = vec![
            "chapter1/file1.md".to_string(),
            "chapter1/subchap/file1.md".to_string(),
        ];

        let expected: &str = &format!(
            "# {}\n\n{} [File1](./file1.md)\n- [Chapter1]()\n\t- [File1](chapter1/file1.md)",
            TITLE, LIST_CHAR
        );

        let book = Chapter::new(TITLE.to_string(), &input);

        assert_eq!(expected, book.get_summary_file("md"));
    }
}
