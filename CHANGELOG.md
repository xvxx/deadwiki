## 0.1.13-dev

- First release with a working GUI mode. Launch it by compiling with
  the `gui` feature and running deadwiki with `-g`:

  cargo run --features gui -- -g

If you don't pass a path to a wiki it'll pop up a "find directory"
dialog. Point to your wiki and get crackin!

Eventually this will (probably) be a menu bar application for macOS
and Linux.
