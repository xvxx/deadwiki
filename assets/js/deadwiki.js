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
    if (e.metaKey && e.key == "[" && !e.shiftKey) {
      // history back: cmd+[ or cmd+left-arrow
      e.preventDefault();
      history.back();
      return;
    } else if (e.metaKey && e.key == "]" && !e.shiftKey) {
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

window.WIKI_PAGES = [];
var first = true;
document.querySelectorAll("#jump-list li a").forEach((el) => {
  if (first) {
    el.closest("li").classList.add("active");
    first = false;
  }
  window.WIKI_PAGES.push({
    name: el.innerText,
    path: el.href.split("/").slice(-1)[0],
  });
});

// jump to page (ctrl-j)
document.querySelector("#jump-pattern").addEventListener("keydown", (e) => {
  // down arrow or ctrl-n
  if (e.keyCode == 40 || (e.keyCode == 78 && e.ctrlKey)) {
    let el = document.querySelector("#jump-list .active");
    var next = el.nextElementSibling;
    while (next && !next.offsetParent) next = next.nextElementSibling;
    if (next && next.offsetParent) {
      el.classList.remove("active");
      next.classList.add("active");
    }
    e.preventDefault();
    e.stopPropagation();
    return false;
  }

  // up arrow or ctrl-p
  if (e.keyCode == 38 || (e.keyCode == 80 && e.ctrlKey)) {
    let el = document.querySelector("#jump-list .active");
    var prev = el.previousElementSibling;
    while (prev && !prev.offsetParent) prev = prev.previousElementSibling;
    if (prev && prev.offsetParent) {
      el.classList.remove("active");
      prev.classList.add("active");
    }
    e.preventDefault();
    e.stopPropagation();
    return false;
  }

  // enter (open page/tag)
  if (e.keyCode == 13) {
    window.location = document.querySelector("#jump-list .active a").href;
    e.preventDefault();
    return false;
  }

  const fuse = new Fuse(window.WIKI_PAGES, { keys: ["name"] });
  const pattern = e.target.value;
  let list = document.querySelectorAll("#jump-list li");
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
      var active = document.querySelector(".active");
      if (active) active.classList.toggle("active");
      el.classList.add("active");
      jumpList.insertBefore(el, jumpList.childNodes[0]);
      el.style.display = "";
    }
  }
});