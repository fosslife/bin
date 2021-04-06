// deno-lint-ignore-file
function create(data, language) {
    return fetch("/", {
        method: "POST",
        headers: { "X-Language": language },
        body: data,
    }).then(function (res) {
        return res.text().then(function (text) {
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

function goto() {
    const line = prompt("Enter line number to goto:");
    const lineNumber = Number(line);
    editor.setPosition({ lineNumber, column: 0 });
    editor.focus();
}

function setLang(e) {
    editor.trigger("Source", "vs.editor.ICodeEditor:1:set-language");
}

function fork() {
    if (isNew) {
        return;
    }
    history.pushState({}, document.title, "/");
    editor.updateOptions({ readOnly: false });
    isNew = true;
}

function save() {
    if (!isNew) {
        return;
    }
    var data = editor.getValue();
    var language = ((editor.getModel() || {})._languageIdentifier || {})
        .language;
    create(data, language)
        .then(function (uuidBytes) {
            var uuid = uuidBytes[0];
            var bytes = uuidBytes[1];
            localStorage.removeItem("paste");
            location.href = "/" + uuid;
            return;
        })
        .catch(function (err) {
            var msg = err.message;
            editor.setValue(editor.getValue() + "\n// Error: " + msg);
        });
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
    editor.onDidChangeModelContent(() => {
        const error = monaco.editor.getModelMarkers({});
        let errordiv = document.querySelector(".error-counter");
        let warndiv = document.querySelector(".warn-counter");
        if (error.length) {
            errordiv.innerHTML = error.length;
            warndiv.innerHTML = error[0].message;
            // console.log();
        } else {
            errordiv.innerHTML = 0;
            warndiv.innerHTML = "";
        }
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
            save();
            return;
        }
        if (e.code === "KeyF" && e.shiftKey) {
            if (isNew) {
                return;
            }
            e.preventDefault();
            fork();
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
