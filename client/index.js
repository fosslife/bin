// import * as monaco from "monaco-editor/esm/vs/editor/editor.api";
import * as monaco from "monaco-editor";
import "./index.css";

let isNew = location.pathname === "/";

const editor = monaco.editor.create(document.getElementById("main"), {
    theme: "vs-dark",
    readOnly: !isNew,
});
editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KEY_S, submit);

async function submit() {
    const code = editor.getValue();
    const url = await fetch("/create", {
        method: "POST",
        headers: {
            "content-type": "application/json",
        },
        body: code,
    })
        .then((d) => d.text())
        .catch(console.error);
    history.pushState({}, "", url);
    const model = editor.getModel();
    monaco.editor.setModelLanguage(model, "typescript")
    editor.updateOptions({
        readOnly: true
    })
    console.log("Submitted", );
}
