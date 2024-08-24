# Bookmark Exporter

[![coverage](https://shields.io/endpoint?url=https://raw.githubusercontent.com/jlyonsmith/bookmark_exporter/main/coverage.json)](https://github.com/jlyonsmith/bookmark_exporter/blob/main/coverage.json)
[![Crates.io](https://img.shields.io/crates/v/bookmark_exporter.svg)](https://crates.io/crates/bookmark_exporter)
[![Docs.rs](https://docs.rs/bookmark_exporter/badge.svg)](https://docs.rs/bookmark_exporter)

This is a basic CLI bookmark exporter tool.

Currently the following browsers are supported:

| Browser | Argument    | Description                                                                                                                                         |
| ------- | ----------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| Firefox | `--firefox` | Firefox uses a SQLite database so *the browser must be closed to do the export.*                                                                    |
| Chrome  | `--chrome`  | Chrome uses a JSON file and the export can be run with the browser open. You may <br/> still want to close it if you have recently added bookmarks. |

The tool has been tested on:

- macOS

If you want additional functionality, please add it and make a pull request.

## Formatting

The tool dumps bookmark information to `stdout` in line pairs; title followed by URL.  You can format the output as you wish.  To format as markdown links, for example, you could do:

```sh
bookmark-exporter --chrome --firefox | tr -d "\"'" | gxargs -d '\n' -L2 printf "[%s](%s)\n" \"$0\" \"$1\"
```

On a Mac this assumes you have installed the GNU version of `xargs`, `gxargs` with `brew install findutils`.
