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
    pub mdheader: bool,
}

impl Chapter {
    pub fn new(name: String, entries: &[String], mdheader: bool) -> Chapter {
        let mut chapter = Chapter {
            name,
            files: vec![],
            chapter: vec![],
            mdheader,
        };

        for entry in entries {
            chapter.add_entry(entry.split('/').collect::<Vec<_>>(), "");
        }

        chapter.sort_contents();
        chapter
    }

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
                    mdheader: false,
                };
                chapter.add_entry(entry[1..].to_owned(), &new_root);
                self.chapter.push(chapter);
            }
        } else {
            self.files.push(new_root)
        }
    }

    fn sort_contents(&mut self) {
        self.files.sort_by(|a, b| {
            let a_key = parse_sort_key(a);
            let b_key = parse_sort_key(b);
            a_key.cmp(&b_key)
        });

        self.chapter.sort_by(|a, b| a.name.cmp(&b.name));

        for child in &mut self.chapter {
            child.sort_contents();
        }
    }

    pub fn get_summary_file(
        &self,
        format: &Format,
        prefered_chapter: &Option<Vec<String>>,
        mdheader: bool,
    ) -> String {
        let indent_level = 0;
        let mut summary: String = "".to_string();
        summary.push_str(&format!("# {}\n\n", self.name));
        match format {
            Format::Md(list_char) => {
                summary += &print_files(&self.files, list_char, indent_level, mdheader)
            }
            Format::Git(list_char) => {
                summary += &print_files(&self.files, list_char, indent_level, mdheader)
            }
        }

        // first prefered chapters (sort)
        if let Some(chapter_names) = prefered_chapter {
            for chapter_name in chapter_names {
                if let Some(chapter) = self
                    .chapter
                    .iter()
                    .find(|c| c.name.to_lowercase() == chapter_name.to_lowercase())
                {
                    summary += &chapter.create_tree_for_summary(&format, indent_level, mdheader);
                }
            }
        }

        for c in &self.chapter {
            if let Some(chapter_names) = prefered_chapter {
                if chapter_names
                    .iter()
                    .map(|n| n.to_lowercase())
                    .any(|x| x == c.name.to_lowercase())
                {
                    continue;
                }
            }

            summary += &c.create_tree_for_summary(&format, indent_level, mdheader);
        }
        summary
    }

    fn create_tree_for_summary(&self, format: &Format, indent: usize, mdheader: bool) -> String {
        let mut summary: String = " ".repeat(4 * indent);
        let list_char = match format {
            Format::Md(c) => c,
            Format::Git(c) => c,
        };

        if let Some(readme) = self
            .files
            .iter()
            .find(|f| f.to_lowercase().ends_with("/readme.md"))
        {
            summary += &format!(
                "{} [{}]({})\n",
                list_char,
                titlecase(&self.name),
                percent_encode_path(readme)
            )
        } else {
            match format {
                Format::Md(_) => {
                    let link = self
                        .infer_chapter_link()
                        .unwrap_or_else(|| format!("{}.md", titlecase(&self.name)));
                    summary.push_str(&format!(
                        "{} [{}]({})\n",
                        list_char,
                        titlecase(&self.name),
                        percent_encode_path(&link)
                    ))
                }
                Format::Git(_) => {
                    summary.push_str(&format!("{} {}\n", list_char, titlecase(&self.name)))
                }
            }
        }

        summary += &print_files(&self.files, list_char, indent + 1, mdheader);

        for c in &self.chapter {
            summary += &c.create_tree_for_summary(&format, indent + 1, mdheader);
        }
        summary
    }

    fn infer_chapter_link(&self) -> Option<String> {
        let first_path = self.first_content_path()?;
        let components: Vec<&str> = first_path.split('/').collect();

        components
            .iter()
            .position(|component| *component == self.name)
            .map(|index| format!("{}.md", components[..=index].join("/")))
    }

    fn first_content_path(&self) -> Option<&str> {
        self.files
            .first()
            .map(String::as_str)
            .or_else(|| self.chapter.iter().find_map(Chapter::first_content_path))
    }
}

fn parse_sort_key(filename: &str) -> (i32, i32) {
    if let Some((vol, chap, _, _)) = parse_filename(filename) {
        (vol, chap)
    } else {
        (i32::MAX, i32::MAX)
    }
}

fn parse_filename(filename: &str) -> Option<(i32, i32, String, String)> {
    let path = Path::new(filename);
    let stem = path.file_stem()?.to_str()?;
    let parts: Vec<&str> = stem.split('.').collect();
    if parts.len() < 3 {
        return None;
    }
    let volume = parts[0].parse::<i32>().ok()?;
    let chapter = parts[1].parse::<i32>().ok()?;
    let title = parts[2..].join(".");
    Some((volume, chapter, title, filename.to_string()))
}

fn print_files(files: &[String], list_char: &char, indent: usize, mdheader: bool) -> String {
    files
        .iter()
        .filter(|f| !f.to_lowercase().ends_with("/readme.md"))
        .map(|f| {
            let title = if mdheader {
                get_first_header(&f).unwrap_or_else(|| get_display_title(f))
            } else {
                get_display_title(f)
            };

            format!(
                "{}{} [{}]({})\n",
                " ".repeat(4 * indent),
                list_char,
                title,
                percent_encode_path(&f)
            )
        })
        .collect::<Vec<String>>()
        .join("")
}

fn get_display_title(file_path: &str) -> String {
    if parse_filename(file_path).is_some() {
        Path::new(file_path)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    } else {
        titlecase(
            &Path::new(file_path)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .replace('_', " "),
        )
    }
}

fn get_first_header(file_path: &str) -> Option<String> {
    let full_path = Path::new(file_path);
    let content = std::fs::read_to_string(full_path).ok()?;
    content
        .lines()
        .find(|line| line.starts_with("# "))
        .map(|line| line.trim_start_matches("# ").trim().to_string())
}

fn percent_encode_path(path: &str) -> String {
    let needs_encoding = path
        .chars()
        .any(|c| c.is_whitespace() || c == '#' || c == '[' || c == ']' || c == '<' || c == '>');

    if needs_encoding {
        format!("<{}>", path)
    } else {
        path.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_filename_test() {
        let filename = "0.1.131313.md";
        let (vol, chap, title, _) = parse_filename(filename).unwrap();
        assert_eq!(vol, 0);
        assert_eq!(chap, 1);
        assert_eq!(title, "131313");
    }

    #[test]
    fn print_files_test() {
        let files = vec![
            "0.1.ABcdc.md".to_string(),
            "0.2.BcDAc.md".to_string(),
            "0.10.bbdac.md".to_string(),
        ];
        let expected = r#"- [0.1.ABcdc](0.1.ABcdc.md)
- [0.2.BcDAc](0.2.BcDAc.md)
- [0.10.bbdac](0.10.bbdac.md)
"#;
        assert_eq!(expected, print_files(&files, &'-', 0, false));
    }

    #[test]
    fn percent_encode_path_test() {
        // No encoding needed
        assert_eq!(
            "normal/path/file.md",
            percent_encode_path("normal/path/file.md")
        );
        // Space needs encoding (angle brackets)
        assert_eq!(
            "<path with spaces/file.md>",
            percent_encode_path("path with spaces/file.md")
        );
        // Hash needs encoding
        assert_eq!(
            "<path#hash/file.md>",
            percent_encode_path("path#hash/file.md")
        );
        // Multiple special chars
        assert_eq!(
            "<path [special]/file.md>",
            percent_encode_path("path [special]/file.md")
        );
    }

    #[test]
    fn md_chapter_links_keep_nested_directory_path() {
        let input = vec!["A/A1/page.md".to_string()];
        let book = Chapter::new("Summary".to_string(), &input, false);

        let expected = r#"# Summary

- [A](A.md)
    - [A1](A/A1.md)
        - [Page](A/A1/page.md)
"#;

        assert_eq!(
            expected,
            book.get_summary_file(&Format::Md('-'), &None, false)
        );
    }
}
