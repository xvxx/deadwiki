window.onload = () => {
  // focus the element with id=focused
  var focused = document.getElementById("focused");
  if (focused && focused.value == "") focused.focus();

  // dbl click wiki content to edit
  var editLink = document.getElementById("edit-link");
  if (editLink) {
    window.addEventListener("dblclick", function () {
      window.location = editLink.href;
    });
  }

  // markdown editor
  var simplemde = new SimpleMDE({
    autofocus: !focused || focused.value != "",
    autoDownloadFontAwesome: false,
    blockStyles: {
      italic: "_",
    },
    indentWithTabs: false,
    renderingConfig: {
      singleLineBreaks: false,
      codeSyntaxHighlighting: true,
    },
    status: false,
    tabSize: 4,
    element: document.getElementById("markdown"),
  });

  // jump-to-page
  const fuse = new Fuse(window.WIKI_PAGES, { keys: ["name"] });
  const pattern = "";
  console.log(fuse.search(pattern));
};

document.onkeydown = (e) => {
  e = e || window.event || {};

  // check if we're running the native app
  if (document.getElementById("main").classList.contains("webview-app")) {
    if (e.metaKey && (e.key == "[" || e.keyCode == 37)) {
      // history back: cmd+[ or cmd+left-arrow
      e.preventDefault();
      history.back();
      return;
    } else if (e.metaKey && (e.key == "]" || e.keyCode == 47)) {
      // history forward: cmd+] or cmd+right-arrow
      e.preventDefault();
      history.forward();
      return;
    }
  }

  // global shortcuts for pages that don't have the editor
  if (!document.getElementById("markdown")) {
    // ctrl-h goes home
    if (e.ctrlKey && e.key == "h") return (window.location = "/");

    // ctrl-n new
    if (e.ctrlKey && e.key == "n") return (window.location = "/new");

    // ctrl-e edit
    var editLink = document.getElementById("edit-link");
    if (editLink)
      if (e.ctrlKey && e.key == "e") return (window.location = editLink.href);

    ////
    // everything after this are shortcuts only for the editor
    return;
  }

  // ESC key to go back when editing
  if (e.keyCode == 27) history.back();

  // CTRL+ENTER to submit when editing
  if ((e.ctrlKey || e.metaKey) && e.keyCode == 13)
    document.getElementById("form").submit();
};
