<img src="/assets/img/rip.gif" alt="R.I.P." height="200" align="left">

# deadwiki

**deadwiki** is a Markdown-powered wiki that uses your filesystem as
its db. This means you can keep your wiki in a git repository and edit
content with your text editor, or read and modify pages `with style`
using its 1990s-era web interface.

---

There are three built-in ways to access your deadwiki:

- Run the local webserver and use the (minimal) HTML UI.
- Run a native WebView app that wraps the UI.
- Just use your filesystem. Regular Markdown files. `cat`, `ls`, etc.

## ~ status ~

Under construction!

_The git `master` may be broken, so make sure you install from
crates.io (see below)._

I use it every day, and I like combining it with other small tools. I
keep a Markdown TODO list in `~/.todo` that I manage on the command
line with a `todo` program, and I have a little scratch pad in
`~/.scratch` that I add links to in a shell using an `s` program,
like:

    $ s https://git.coolstuff.com/some/repo

With deadwiki, I symlinked both of those files into my `~/.deadwiki`
dir and can browse them using a fancy, 1990s-era HTML interface.

## ~ installation ~

Okay, so how do you get started? You just need `grep`, which you
probably already have, and [cargo], which is usually pretty easy to
install.

Once you've got both of them you can install it with:

    cargo install deadwiki

That'll give you a handy dandy `dead` CLI tool, if everything is setup
and `~/.cargo/bin` is in your `$PATH`. You should now be able to run
`dead -h` to see the possibilities.

## ~ getting started ~

To begin, create an empty directory or find one already populated with
`.md` files. This is your deadwiki. Simply point the CLI utility at it
to get going:

    $ dead my-wiki-dir/
    -> deadwiki serving my-wiki-dir/ at http://0.0.0.0:8000

Now visit http://0.0.0.0:8000/ in your browser! (Or don't. It's up to
you.)

You can edit wiki pages locally with something like `vim`, or by using
the web UI. Edits show up on the next page load, as do new pages -
there is no database and no fancy pantsy caching. Just you, your
filesystem, and a dream.

In addition to [CommonMark], Markdown files can link to each other by
putting the `[Page Name]` in brackets. Like most wikis, it'll either
be a link to the actual page or a link to create it.

deadwiki also includes support for `#hashtags`. Any hashtag appearing
in wiki text will be linked to a search page that lists all wiki pages
containing that hashtag.

Finally, if you want to sync your wiki automatically, there is some
_very basic_ git support. Basically, if you start the `dead`
program with the `-s` or `--sync` flag and point it at an existing git
repository, it'll do this every 30 seconds or so:

    git add .
    git commit -am update
    git pull origin master
    git push origin master

Like I said, super basic! But it works, and it's nice that it syncs
changes you make even outside of the web UI.

## ~ keyboard shortcuts ~

There are two modes: browsing and editing. Editing is powered by
[SimpleMDE] and includes all its default shortcuts (shown below), plus
a few deadwiki-specific shortcuts.

Browsing mode includes a few keyboard shortcuts to make navigation
quicker and more nimble.

### Browse Mode

| **Shortcut**   | **Notes**                    |
| -------------- | ---------------------------- |
| `Double Click` | Enter edit or create mode    |
| `Ctrl-h`       | Go to the home page          |
| `Ctrl-j`       | Jump to page (fuzzy finder)  |
| `Ctrl-n`       | Go to the "new" page         |
| `Ctrl-e`       | Open editor for current page |
| `i`            | Insert mode: Edit or New     |

### Edit Mode

| **Shortcut**  | **Notes**                |
| ------------- | ------------------------ |
| `ESC`         | Exits edit mode          |
| `Ctrl Enter`  | Submits your edits       |
| `Cmd Enter`   | Same                     |
| `Cmd-'`       | Toggle Blockquote        |
| `Cmd-B`       | Toggle Bold              |
| `Cmd-E`       | Clean Block              |
| `Cmd-H`       | Toggle Heading (Smaller) |
| `Cmd-I`       | Toggle Italic            |
| `Cmd-K`       | Draw Link                |
| `Cmd-L`       | Toggle Unordered List    |
| `Cmd-P`       | Toggle Preview           |
| `Cmd-Alt-C`   | Toggle Code Block        |
| `Cmd-Alt-I`   | Draw Image               |
| `Cmd-Alt-L`   | Toggle Ordered LIST      |
| `Shift-Cmd-H` | Toggle Heading (Bigger)  |
| `F9`          | Toggle Side-By-Side      |
| `F11`         | Toggle Fullscreen        |

## ~ gui mode ~

There is a "native" app that uses your system's WebKit to view the
local webserver, which it starts and manages.

You can start it by using `make gui` to build a `dead` binary, then
passing the `-g` flag to it:

    $ make gui
    $ ./dead -g -p 8001 wiki/

## ~ hacking ~

The code is in pretty rough shape right now, as this is mostly a
prototype-in-progress. But you can hack on it pretty easily with
[cargo]:

    $ git clone https://github.com/xvxx/deadwiki
    $ cd deadwiki
    $ cargo run wiki/

There's a basic wiki included that shows off some features.

## ~ future features ~

- `--read-only` mode, so i can have a copy i can view anywhere
- mobile-friendly CSS
- search (probably just `grep`)
- `--gopher`: serve wiki pages over gopher too (probably using [phd])
- `*.css` in wiki dir gets included
- homebrew package, AUR package
- `brew services` for running on osx, `systemd` for arch

## ~ philosophy ~

- no database
- text editor/plain text friendly
- prefer server-side rendering
- take your data with you (scm friendly)
- lean on standard UNIX commands (find, grep)
- js only for user input (keyboard shortcuts, markdown editor, finder)
- no js frameworks/helpers
- build time matters (42 crates currently, ~8s release ~6s debug)

## ~ screenies ~

| ![screenie1](/assets/img/screenie1.jpeg) | ![screenie1](/assets/img/screenie2.jpeg) |
| :--------------------------------------: | :--------------------------------------: |
|        Rendering Markdown. `Wow.`        |       Editing Markdown. `Amazing.`       |

## ~ bug reports ~

Please direct all known and unknown (suspected) bugs to this URL:

- https://github.com/xvxx/deadwiki/issues/new

[cargo]: https://rustup.rs
[simplemde]: https://simplemde.com/
[keys]: https://github.com/sparksuite/simplemde-markdown-editor#keyboard-shortcuts
[commonmark]: https://commonmark.org/
[phd]: https://github.com/xvxx/phd
