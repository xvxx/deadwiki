<img src="./static/rip.gif" alt="R.I.P." height="200" align="left">

# deadwiki

**deadwiki** is a Markdown-powered wiki that uses your filesystem as
its db. This means you can keep your wiki in a git repository and edit
content with your text editor, or read and modify pages _with style_
using its 1990s-era web interface.

---

There are three built-in ways to access your deadwiki:

- Run the local webserver and use the static HTML UI.
- Run a native WebView app that wraps the UI.
- Just use your filesystem. Regular Markdown files. `cat`, `ls`, etc.

## ~ status ~

Very barebones, no native app yet. Under construction!

## ~ getting started ~

To begin, create an empty directory or find one already populated with
`.md` files. This is your deadwiki. Simply point the CLI utility at it
to get going:

    dead my-wiki-dir/
    -> deadwiki serving my-wiki-dir/ at http://0.0.0.0:8000

Now visit that URL in your browser! (Or don't. It's up to you.)

You can edit wiki pages locally with something like `vim`, or by using
the web UI. Edits show up instantly, as do new pages - there is no
database and no fancy pantsy caching. Just you, your filesystem, and a
dream.

## ~ installation ~

Hey, how do I get that handy dandy `dead` CLI utility? With [cargo]:

    cargo install deadwiki

Now you should be able to run `dead -h` to see the possibilities.

[cargo]: https://rustup.rs
