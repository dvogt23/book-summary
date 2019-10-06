# book-summary

![book-summary-tests build status](https://github.com/dvogt23/book-summary/workflows/test/badge.svg)

> Automatically creates a SUMMARY.md file for your book (`mdbook/gitbook`)

Similar to the `npm` version of the auto-summary tool [gh/imfly/gitbook-summary](https://github.com/imfly/gitbook-summary), here is a `rust` version.

My initial intention is to get the chapters sorted without having to rename the chapter folders with a prefix number. The `-s` option takes the name of the chapters wich should come first. I use it in my personal notes repository: [notes](https://github.com/dvogt23/notes)

## Installation

OS X & Linux:

```sh
git clone https://github.com/dvogt23/book-summary.git
cd book-summary
make install
```

## Usage example

```sh
# create a SUMMARY.md file with custom sort in mdBook format
$ book-summary -n ./notes --sort tech,personal
```

```sh
book-summary 0.1.0

USAGE:
    book-summary [FLAGS] [OPTIONS]

FLAGS:
    -d, --debug       Activate debug mode
    -h, --help        Prints help information
    -m, --mdheader    Title from md file header?
    -V, --version     Prints version information
    -v, --verbose     Verbose mode (-v, -vv, -vvv)

OPTIONS:
    -f, --format <format>            Format md/git book [default: md]
    -n, --notesdir <notesdir>        Notes dir where to parse all your notes from [default: ./]
    -o, --outputfile <outputfile>    Output file [default: SUMMARY.md]
    -s, --sort <sort>...             Default sort by asc, but first start with following chapters
    -t, --title <title>              Title for summary [default: Summary]
```

## Contributing

Feel free to open a pull request or an issue to contribute to this project.

## Authors

* **Dimitrij Vogt** - *Initial work* - [gh/dvogt23](https://github.com/dvogt23)

See also the list of [contributors](https://github.com/dvogt23/book-summary/contributors) who participated in this project.

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.
