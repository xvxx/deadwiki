window.onload = () => {
  // focus the element with id=focused
  var focused = document.querySelector("#focused");
  if (focused && focused.value == "") focused.focus();
  // or class=focused
  var focused = document.querySelector(".focused");
  if (focused && focused.value == "") focused.focus();

  // markdown editor
  var el = document.querySelector("#markdown");
  if (el) {
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
      element: el,
    });
  }
};

document.onkeydown = (e) => {
  e = e || window.event || {};
  let name = document.activeElement.tagName;

  // editor shortcuts pages that don't have the editor
  if (document.querySelector("#markdown")) {
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
  } else if (name != "INPUT" && name != "TEXTAREA") {
    // ignore keydown if input or textarea is focused

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

    // ctrl-n new / i insert
    if (e.key == "i" || (e.ctrlKey && e.key == "n")) {
      e.preventDefault();
      let newLink = document.querySelector("#new-link");
      if (newLink) {
        return (window.location = newLink.href);
      } else {
        return (window.location = "/new");
      }
    }

    // ctrl-e edit / i insert mode
    var editLink = document.querySelector("#edit-link");
    if (editLink && ((e.ctrlKey && e.key == "e") || e.key == "i")) {
      e.preventDefault();
      return (window.location = editLink.href);
    }

    ////
    // everything after this are shortcuts only for the editor
    return;
  }
};
