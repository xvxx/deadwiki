window.onload = () => {
  // focus the element with id=focused
  var focused = document.getElementById("focused");
  if (focused && focused.value == "") focused.focus();
  // or class=focused
  var focused = document.getElementsByClassName("focused");
  if (focused[0] && focused[0].value == "") focused[0].focus();

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
};

document.onkeyup = (e) => {
  // jump-to-page js
  let jumpInput = document.getElementById("jump-pattern");
  if (jumpInput) {
    const fuse = new Fuse(window.WIKI_PAGES, { keys: ["name"] });
    const pattern = jumpInput.value;
    let list = document.getElementById("jump-list");
    console.log(jumpInput.value);
    if (pattern == "") {
      for (var i = 0; i < list.children.length; i++) {
        let el = list.children[i];
        el.style.display = "";
      }
    } else {
      let matches = fuse.search(pattern);
      for (var i = 0; i < list.children.length; i++) {
        let el = list.children[i];
        el.style.display = "none";
      }
      for (var i = 0; i < matches.length; i++) {
        document.getElementById("jump-" + matches[i].refIndex).style.display =
          "";
      }
    }
  }
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
