use std::path::Path;

#[derive(Debug, PartialEq)]
pub struct Chapter {
    pub name: String,
    pub files: Vec<String>,
    pub chapter: Vec<Chapter>,
}

impl Chapter {
    pub fn new(name: String, entries: &Vec<String>) -> Chapter {
        let mut chapter = Chapter {
            name,
            files: vec![],
            chapter: vec![],
        };

        chapter.add_entries(entries);

        chapter
    }

    pub fn add_entries(&mut self, entries: &Vec<String>) {
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

    pub fn get_summary_file(&self, format: &str) -> String {
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
        summary.push_str(&format!("# {}\n\n", self.name));

        // add book files
        summary.push_str(&get_files_md(".", &self.files, &list_char));
        summary.push_str("\n");

        // add chapter with files
        for chapter in self.chapter.iter() {
            if chapter.files.contains(&"README.md".to_string()) {
                summary.push_str(&format!(
                    "{} [{}]({}/README.md)",
                    list_char,
                    make_title_case(&chapter.name),
                    chapter.name
                ));
                summary.push_str("\n\t");
            } else {
                summary.push_str(&format!(
                    "{} [{}]()",
                    list_char,
                    make_title_case(&chapter.name)
                ));
                summary.push_str("\n\t");
            }

            summary.push_str(&get_files_md(&chapter.name, &chapter.files, &list_char));
        }

        summary
    }
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

    // println!(
    // "path: {:?}, parent: {:?}, filename; {:?}",
    // full_path, parent, file_name
    // );

    let mut c = file_name.to_str().unwrap().chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().chain(c).collect(),
    }
}
