window.onload = () => {
  // focus the element with id=focused
  var focused = document.querySelector("#focused");
  if (focused && focused.value == "") focused.focus();
  // or class=focused
  var focused = document.querySelector(".focused");
  if (focused && focused.value == "") focused.focus();

  // dbl click wiki content to edit
  var editLink = document.querySelector("#edit-link");
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
    element: document.querySelector("#markdown"),
  });
};

document.onkeyup = (e) => {
  // jump-to-page js
  let jumpInput = document.querySelector("#jump-pattern");
  if (jumpInput) {
    const fuse = new Fuse(window.WIKI_PAGES, { keys: ["name"] });
    const pattern = jumpInput.value;
    let list = document.querySelectorAll("#jump-list li");
    console.log(list);
    if (pattern == "") {
      list.forEach((el) => (el.style.display = ""));
    } else {
      let matches = fuse.search(pattern);
      list.forEach((el) => (el.style.display = "none"));
      for (var i = matches.length - 1; i >= 0; i--) {
        let match = matches[i];
        let el = document.querySelector("#jump-" + match.refIndex);
        let jumpList = document.querySelector("#jump-list");
        jumpList.removeChild(el);
        jumpList.insertBefore(el, jumpList.childNodes[0]);
        el.style.display = "";
      }
    }
  }
};

document.onkeydown = (e) => {
  e = e || window.event || {};

  // check if we're running the native app
  if (document.querySelector("#main.webview-app")) {
  }

  // history navigation
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

  // jump-to-page js
  let jumpInput = document.querySelector("#jump-pattern");
  if (jumpInput && e.keyCode == 13) {
    let jumpList = document.querySelector("#jump-list");
    for (var i = 0; i < jumpList.children.length; i++) {
      let el = jumpList.children[i];
      if (el.style.display != "none") {
        window.location = el.children[0].href;
        e.preventDefault();
        return;
      }
    }
  }

  // global shortcuts for pages that don't have the editor
  if (!document.querySelector("#markdown")) {
    // ctrl-h goes home
    if (e.ctrlKey && e.key == "h") {
      e.preventDefault();
      return (window.location = "/");
    }

    // ctrl-j jump to page
    if (e.ctrlKey && e.key == "j") {
      e.preventDefault();
      return (window.location = "/jump");
    }

    // ctrl-n new
    if (e.ctrlKey && e.key == "n") {
      e.preventDefault();
      return (window.location = "/new");
    }

    // ctrl-e edit
    var editLink = document.querySelector("#edit-link");
    if (editLink)
      if (e.ctrlKey && e.key == "e") {
        e.preventDefault();
        return (window.location = editLink.href);
      }

    ////
    // everything after this are shortcuts only for the editor
    return;
  }

  // ESC key to go back when editing
  if (e.keyCode == 27) {
    e.preventDefault();
    return history.back();
  }

  // CTRL+ENTER to submit when editing
  if ((e.ctrlKey || e.metaKey) && e.keyCode == 13) {
    e.preventDefault();
    return document.querySelector("#form").submit();
  }
};
