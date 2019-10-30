use std::path::Path;
use std::str::FromStr;
use std::string::ParseError;
use titlecase::titlecase;

#[derive(Debug, PartialEq)]
pub enum Format {
    Md(char),
    Git(char),
}

impl FromStr for Format {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "md" => Ok(Format::Md('-')),
            "git" => Ok(Format::Git('*')),
            _ => panic!("Error: Invalid format {}", s),
        }
    }
}

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

        for entry in entries {
            chapter.add_entry(entry.split("/").collect::<Vec<_>>(), "");
        }

        chapter
    }

    // This is a recursive function to add new chapters and files to an existing chapter.
    fn add_entry(&mut self, entry: Vec<&str>, root: &str) {
        let new_root = match root {
            "" => entry[0].to_string(),
            _ => format!("{}/{}", root, entry[0]),
        };

        if entry.len() > 1 {
            if let Some(chapter) = self.chapter.iter_mut().find(|c| c.name == entry[0]) {
                chapter.add_entry(entry[1..].to_owned(), &new_root)
            } else {
                let mut chapter = Chapter {
                    name: entry[0].to_string(),
                    files: vec![],
                    chapter: vec![],
                };
                chapter.add_entry(entry[1..].to_owned(), &new_root);

                self.chapter.push(chapter);
            }
        } else {
            self.files.push(new_root)
        }
    }

    pub fn get_summary_file(&self, format: &Format, prefered_chapter: &Option<Vec<String>>) -> String {
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

        let indent_level = 0;
        let mut summary: String = "".to_string();
        summary.push_str(&format!("# {}\n\n", self.name));
        match format {
            Format::Md(list_char) => summary += &print_files(&self.files, list_char, indent_level),
            Format::Git(list_char) => summary += &print_files(&self.files, list_char, indent_level),
        }

        // first prefered chapters (sort)
        if let Some(chapter_names) = prefered_chapter {
            for chapter_name in chapter_names {
                if let Some(chapter) = self
                    .chapter
                    .iter()
                    .find(|c| c.name.to_lowercase() == chapter_name.to_lowercase())
                {
                    summary += &chapter.create_tree_for_summary(&format, indent_level);

                    // match format {
                        // Format::Md(list_char) => summary += &chapter.create_tree_for_summary(list_char, indent_level),
                        // Format::Git(list_char) => summary += &chapter.create_tree_for_summary(list_char, indent_level),
                    // }
                }
            }
        }

        for c in &self.chapter {
            if let Some(chapter_names) = prefered_chapter {
                if chapter_names
                    .iter()
                    .map(|n| n.to_lowercase())
                    .collect::<Vec<String>>()
                    .contains(&c.name.to_lowercase())
                {
                    continue;
                }
            }

            summary += &c.create_tree_for_summary(&format, indent_level);

            // match format {
                // Format::Md(list_char) => summary += &c.create_tree_for_summary(list_char, indent_level),
                // Format::Git(list_char) => summary += &c.create_tree_for_summary(list_char, indent_level),
            // }
        }
        summary
    }

    fn create_tree_for_summary(&self, format: &Format, indent: usize) -> String {
        let mut summary: String = " ".repeat(4 * indent);
        let list_char = match format {
            Format::Md(c) => c,
            Format::Git(c) => c,
        };

        if let Some(readme) = self
            .files
            .iter()
            .filter(|f| f.to_lowercase().ends_with("/readme.md"))
            .nth(0)
        {
            summary += &format!(
                "{} [{}]({})\n",
                list_char,
                make_title_case(&self.name),
                readme
            )
        } else {
            match format {
                Format::Md(_) => summary.push_str(&format!(
                        "{} [{}](#)\n",
                        list_char,
                        make_title_case(&self.name)
                )),
                Format::Git(_) => summary.push_str(&format!(
                        "{} {}\n",
                        list_char,
                        make_title_case(&self.name)
                )),
            }
        }

        summary += &print_files(&self.files, list_char, indent + 1);

        for c in &self.chapter {
            summary += &c.create_tree_for_summary(&format, indent + 1);
        }
        summary
    }
}

fn print_files(files: &Vec<String>, list_char: &char, indent: usize) -> String {
    files
        .iter()
        .filter(|f| !f.to_lowercase().ends_with("/readme.md"))
        .map(|f| {
            format!(
                "{}{} [{}]({})\n",
                " ".repeat(4 * indent),
                list_char,
                make_title_case(Path::new(&f).file_stem().unwrap().to_str().unwrap()),
                &f
            )
        })
        .collect::<Vec<String>>()
        .join("")
}

fn make_title_case(name: &str) -> String {
    titlecase(
        &name
            .chars()
            .skip_while(|c| !c.is_alphabetic())
            .map(|c| if c == '_' { ' ' } else { c })
            .collect::<String>(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn titlecase_test() {
        assert_eq!("Chapter 1", make_title_case("1-chapter_1"));
        assert_eq!("Chapter 23", make_title_case("chapter_23"));
    }

    #[test]
    fn file_print_test() {
        let expected = r#"- [WritingIsGood](part1/WritingIsGood.md)
- [GitbookIsNice](part1/GitbookIsNice.md)
"#;
        let input = vec![
            "part1/README.md".to_string(),
            "part1/WritingIsGood.md".to_string(),
            "part1/GitbookIsNice.md".to_string(),
        ];
        assert_eq!(expected, print_files(&input, &'-', 0));
    }
}
