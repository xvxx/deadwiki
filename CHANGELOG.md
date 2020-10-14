## 0.1.23-dev

- We've cut down the number of dependencies from **42** to **28**,
  shaving a few precious seconds off build time in the process.
- We've done a bit of internal refactoring. There may be (more) bugs.


## 0.1.22

- Bugfix release.

## 0.1.21

- Added `i` keyboard shortcut to edit the current page.
- The "new" form now prefills the title with the current directory.
- Fixed `cargo install deadwiki`.

## 0.1.20

- "Recently edited pages" page added.
- "Jump to page" (`ctrl-j`) now lets you jump to `#hashtags`.
- "Jump to page" results can now be selected using
  `up`/`down`/`ctrl-n`/`ctrl-p` keyboard shortcuts.
- `make install` will build and install the `dead` binary to
  `$PREFIX/bin`, where `$PREFIX` defaults to `$HOME` (`~/bin`).

## 0.1.19

This is a small release that upgrades Vial, fixing a few minor bugs.

## 0.1.18

Jump-to-page via fuzzy finder is now live! Use `ctrl-j` to open the
menu and start typing a page title. This will gain functionality in
the future.

This release also switches deadwiki to a new backend,
[Vial](https://vial.sh). **Vial** is a micro micro-framework for Rust.
This change has cut the dependency count from 72 to 42 and release
compile time from ~22s to ~8s on my machine:

https://github.com/xvxx/deadwiki/commit/c7b844a90dc433703d64059ce7de5bebc5d4fd8f

Enjoy!

## 0.1.17

This release adds support for #hashtags! Very simple: any #hashtag
in wiki content will get turned into a link that takes you to a
search page. The search page will display all the wiki pages that
include the hashtag.

Nothing fancy like searching for more than one hashtag at a time.
Just the basics. Enjoy!

## 0.1.16

Another small UI tweak: when the default page is empty, it gives you a
hint about what it's for.

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
