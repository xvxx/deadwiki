## 0.1.15

This release adds a slight UI tweak - wiki pages in subdirectories are
now swaddled in `<details>`:

![details](https://user-images.githubusercontent.com/41523880/82960235-bdef0a80-9f6e-11ea-8f85-27752a9462a1.png)

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
