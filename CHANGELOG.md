## 0.1.14

This release includes new keyboard shortcuts for browsing around the
wiki:

- `Ctrl-h`: Go to home page
- `Ctrl-e`: Open edit mode for the current wiki page
- `Ctrl-n`: Go to "create new wiki page" page

You can also double click on a wiki page to enter edit or create mode.

Enjoy!

## 0.1.13

- First release with a working GUI mode. Launch it by compiling with
  the `gui` feature and running deadwiki with `-g`:

  cargo run --features gui -- -g

If you don't pass a path to a wiki it'll pop up a "find directory"
dialog. Point to your wiki and get crackin!

Eventually this will (probably) be a menu bar application for macOS
and Linux.
