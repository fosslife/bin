// import * as monaco from "monaco-editor/esm/vs/editor/editor.api";
import * as monaco from "monaco-editor";
import "./index.css";

const editor = monaco.editor.create(document.getElementById("main"), {
    theme: "vs-dark",
});
editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KEY_S, submit);

async function submit() {
    const code = editor.getValue();
    const res = await fetch("/create", {
        method: "POST",
        headers: {
            "content-type": "application/json",
        },
        body: code,
    })
        .then((d) => d.text())
        .catch(console.error);
    console.log("Submitted", res);
}
