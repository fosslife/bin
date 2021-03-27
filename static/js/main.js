// deno-lint-ignore-file
function create(data, language) {
  return fetch("/", {
    method: "POST",
    headers: { "X-Language": language },
    body: data,
  }).then(function (res) {
    return res.text().then(function (text) {
      console.log("returned", text);
      if (res.status !== 200) {
        throw new Error(text);
      }
      var uuidBytes = text.split(" ");
      var uuid = uuidBytes[0];
      var bytes = uuidBytes[1];
      return [uuid, bytes];
    });
  });
}

// const langs = [
//   "ABAP",
//   "APEX",
//   "AZCLI",
//   "BAT",
//   "CAMELIGO",
//   "CLOJURE",
//   "COFFEE",
//   "CPP",
//   "CSHARP",
//   "CSP",
//   "CSS",
//   "DART",
//   "DOCKERFILE",
//   "ECL",
//   "FSHARP",
//   "GO",
//   "GRAPHQL",
//   "HANDLEBARS",
//   "HCL",
//   "HTML",
//   "INI",
//   "JAVA",
//   "JAVASCRIPT",
//   "JULIA",
//   "KOTLIN",
//   "LESS",
//   "LEXON",
//   "LUA",
//   "M3",
//   "MARKDOWN",
//   "MIPS",
//   "MSDAX",
//   "MYSQL",
//   "OBJECTIVE-C",
//   "PASCAL",
//   "PASCALIGO",
//   "PERL",
//   "PGSQL",
//   "PHP",
//   "POSTIATS",
//   "POWERQUERY",
//   "POWERSHELL",
//   "PUG",
//   "PYTHON",
//   "R",
//   "RAZOR",
//   "REDIS",
//   "REDSHIFT",
//   "RESTRUCTUREDTEXT",
//   "RUBY",
//   "RUST",
//   "SB",
//   "SCALA",
//   "SCHEME",
//   "SCSS",
//   "SHELL",
//   "SOLIDITY",
//   "SOPHIA",
//   "SQL",
//   "ST",
//   "SWIFT",
//   "SYSTEMVERILOG",
//   "TCL",
//   "TWIG",
//   "TYPESCRIPT",
//   "VB",
//   "XML",
//   "YAML",
// ];

var main = document.querySelector("#main");
var isNew = location.pathname === "/";

function setLang(e) {
  editor.trigger("Source", "vs.editor.ICodeEditor:1:set-language");
}

require(["vs/editor/editor.main"], function () {
  if (!(main instanceof HTMLElement)) {
    return;
  }
  var editor = monaco.editor.create(main, {
    renderWhitespace: "all",
    theme: "vs-dark",
    wordWrap: "on",
    readOnly: !isNew,
    theme: "vs-dark",
  });
  window.editor = editor;
  window.addEventListener("resize", () => {
    editor.layout();
  });
  // Get cursor
  editor.onDidChangeCursorPosition((e) => {
    document.getElementById(
      "lines"
    ).innerText = `Ln ${e.position.lineNumber}, Col ${e.position.column}`;
  });
  document.addEventListener("keydown", function (e) {
    if (!(e.ctrlKey || e.metaKey)) {
      return;
    }
    if (e.code === "KeyS") {
      if (!isNew) {
        return;
      }
      e.preventDefault();
      var data = editor.getValue();
      var language = ((editor.getModel() || {})._languageIdentifier || {})
        .language;
      console.log(
        "Got language",
        language,
        monaco.editor.model,
        editor._configuration._rawOptions.language
      );
      create(data, language)
        .then(function (uuidBytes) {
          var uuid = uuidBytes[0];
          var bytes = uuidBytes[1];
          console.log(uuid, bytes);
          localStorage.removeItem("paste");
          location.href = "/" + uuid;
          return;
        })
        .catch(function (err) {
          var msg = err.message.startsWith("File already exists: ")
            ? "File already exists at " +
              location.href +
              err.message.split(" ").pop()
            : err.message;
          editor.setValue(editor.getValue() + "\n// Error: " + msg);
        });
      return;
    }
    if (e.code === "KeyF" && e.shiftKey) {
      if (isNew) {
        return;
      }
      e.preventDefault();
      history.pushState({}, document.title, "/");
      editor.updateOptions({ readOnly: false });
      isNew = true;
      return;
    }
    localStorage.setItem("paste", editor.getValue());
  });
  editor.addAction({
    id: "set-language",
    label: "Set language",
    keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.KEY_L],
    run: function (ed) {
      var lang = prompt("Set language");
      var model = ed.getModel();
      if (!model || !lang) {
        return;
      }
      monaco.editor.setModelLanguage(model, lang);
      localStorage.setItem("language", lang);
      document.getElementById("setlang").innerText = lang;
    },
  });
  if (!isNew) {
    var fileLang = location.pathname.split(".");
    var file = fileLang[0];
    var lang =
      fileLang[1] ||
      document
        .querySelector('meta[http-equiv="X-Language"]')
        .getAttribute("content");
    console.log("fetching paste", file, lang);
    if (lang) {
      var model = editor.getModel();
      if (model) {
        monaco.editor.setModelLanguage(model, lang);
      }
    }
    editor.setValue("// Loading...");
    fetch(file, {
      headers: {
        Accept: "text/plain",
      },
    })
      .then(function (res) {
        return res.text();
      })
      .then(function (text) {
        console.log("Fetched", text);
        editor.setValue(text);
      });
  } else {
    var stored = localStorage.getItem("paste");
    if (stored) {
      editor.setValue(stored);
    }
    var lang = localStorage.getItem("language");
    var model = editor.getModel();
    if (lang) {
      if (model) {
        monaco.editor.setModelLanguage(model, lang);
      }
    }
  }
});
