use std::env;
use std::error::Error;
use std::fmt;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

use std::path::PathBuf;
use structopt::StructOpt;

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
    #[structopt(
        name = "outputfile",
        short,
        long,
        default_value = "SUMMARY.md",
        parse(from_os_str)
    )]
    outputfile: PathBuf,

    /// Notes dir where to parse all your notes from
    #[structopt(name = "notesdir", short, long, default_value = "./")]
    dir: PathBuf,
}

#[derive(Debug, PartialEq)]
struct Chapter {
    name: String,
    files: Vec<String>,
    chapter: Vec<Chapter>,
}

impl Chapter {
    fn new(name: String, entries: &Vec<String>) -> Chapter {
        let mut chapter = Chapter {
            name,
            files: vec![],
            chapter: vec![],
        };

        chapter.add_entries(entries);

        chapter
    }

    fn add_entries(&mut self, entries: &Vec<String>) {
        for entry in entries.into_iter() {
            if entry.contains('/') {
                let splits: Vec<&str> = entry.split('/').collect();
                let chapter: Option<&mut Chapter>;

                let chapter_exists = self.chapter.iter().any(|c| c.name == splits[0].to_string());

                if chapter_exists {
                    chapter = self
                        .chapter
                        .iter_mut()
                        .find(|c| c.name == splits[0].to_string());
                    if let Some(chapter) = chapter {
                        chapter.files.push(splits[splits.len() - 1].to_string());
                    }
                } else {
                    self.chapter.push(Chapter {
                        name: splits[0].to_string(),
                        files: vec![splits[splits.len() - 1].to_string()],
                        chapter: vec![],
                    })
                }
            } else {
                self.files.push(entry.to_string());
            }
        }
    }
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

    let entries = match get_dir(&opt.dir) {
        Ok(e) => e,
        Err(err) => panic!(err),
    };

    //    let book = create_chapter(&entries, opt.title);

    let book = Chapter::new(opt.title, &entries);

    println!("BOOK: {:#?}", book);

    // sort_filelist(&mut entries, opt.sort.as_ref());
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn get_dir(dir: &PathBuf) -> Result<Vec<String>> {
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
        let entry = direntry
            .path()
            .to_str()
            .unwrap()
            .chars()
            .skip(dir.to_str().unwrap().len() + 1)
            .collect::<String>();
        if !entry.is_empty() && (entry.contains("/") || entry.contains(".md")) {
            entries.push(entry);
        }
    }
    Ok(entries)
}

fn get_summary_file(book: Chapter, format: &str) -> String {
    // create markdown summary file
    /*
    gitbook format:
    # Summary

    * [First page's title](page1/README.md)
        * [Some child page](page1/page1-1.md)
        * [Some other child page](part1/page1-2.md)
    * [Second page's title](page2/README.md)
        * [Some child page](page2/page2-1.md)
        * [Some other child page](part2/page2-2.md)

    mdbook format:
    # Summary

    - [mdBook](README.md)
    - [Command Line Tool](cli/README.md)
        - [init](cli/init.md)
        - [build](cli/build.md)
        - [watch](cli/watch.md)
        - [serve](cli/serve.md)
        - [test](cli/test.md)
        - [clean](cli/clean.md)
    */

    let mut summary: String = "".to_string();
    let list_char = match format {
        "md" => '-',
        "git" => '*',
        _ => ' ',
    };

    // add title
    summary.push_str(&format!("# {}\n\n", book.name));

    // add book files
    summary.push_str(&get_files_md(".", &book.files, &list_char));
    summary.push_str("\n");

    // add chapter with files
    for chapter in book.chapter.into_iter() {
        if chapter.files.contains(&"README.md".to_string()) {
            summary.push_str(&format!(
                "{} [{}]({}/README.md)",
                list_char,
                make_title_case(&chapter.name),
                chapter.name
            ));
            summary.push_str("\n\r");
        } else {
            summary.push_str(&format!(
                "{} [{}]()",
                list_char,
                make_title_case(&chapter.name)
            ));
            summary.push_str("\n\r");
        }

        summary.push_str(&get_files_md(&chapter.name, &chapter.files, &list_char));
    }

    summary
}

fn get_files_md(path: &str, files: &Vec<String>, list_char: &char) -> String {
    let mut output: String = "".to_string();

    for file in files {
        output.push_str(&format!(
            "{} [{}]({})",
            list_char,
            get_title_capture(&file),
            format!("{}/{}", path, file)
        ));
    }

    output
}

fn make_title_case(name: &str) -> String {
    let mut c = name.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}

fn get_title_capture(path: &str) -> String {
    let full_path = Path::new(path);
    let parent = full_path.parent().unwrap();
    let file_name = full_path.file_stem().unwrap();
    let extension = full_path.extension();

    println!(
        "path: {:?}, parent: {:?}, filename; {:?}",
        full_path, parent, file_name
    );

    let mut c = file_name.to_str().unwrap().chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
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
        assert_eq!(expected, get_dir(&PathBuf::from(r"./examples/book")));
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
                files: vec!["file1.md".to_string()],
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
                files: vec!["file1.md".to_string()],
                chapter: vec![Chapter {
                    name: "subchap".to_string(),
                    files: vec!["file1.md".to_string()],
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

        assert_eq!(expected, get_summary_file(book, "md"));
    }

    #[test]
    fn md_output_onechapter_test() {
        // only one file
        let input: Vec<String> = vec!["file1.md".to_string(), "chapter1/file1.md".to_string()];

        let expected: &str = &format!(
            "# {}\n\n{} [File1](./file1.md)\n- [Chapter1]()\n\r- [File1](chapter1/file1.md)",
            TITLE, LIST_CHAR
        );

        let book = Chapter::new(TITLE.to_string(), &input);

        assert_eq!(expected, get_summary_file(book, "md"));
    }
}
