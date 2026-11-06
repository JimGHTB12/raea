// Tiptap (ProseMirror) on contenteditable (docs/ts-native.feature P14e; the feed's editors are): type a
// paragraph, Enter, Mod-B and type bold, double click a word and delete it, arrows back into the bold mark and type,
// then undo. The document Tiptap holds is shown after each step, so a difference in what the editing did shows as text.
// proves: html.global_attributes.contenteditable, api.Range, api.Document.caretPositionFromPoint, api.Document.selectionchange_event
import { EditorContent, useEditor } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import { useState } from "react";
export const steps = [
  { click: "pm", at: [0.5, 0.15] },
  { text: "Hello world" },
  { key: "Enter" },
  { key: "b", mods: ["meta"] },
  { text: "bold" },
  { key: "b", mods: ["meta"] },
  { text: " plain" },
  // "world" on the first line: two clicks pick it, Backspace deletes it
  { click: "pm", at: [0.2, 0.15], clicks: 2 },
  { key: "Backspace" },
  // to the end of the second paragraph, then back across " plain" into "bold"
  { key: "ArrowDown" },
  { key: "End" },
  { key: "ArrowLeft" },
  { key: "ArrowLeft" },
  { key: "ArrowLeft" },
  { key: "ArrowLeft" },
  { key: "ArrowLeft" },
  { key: "ArrowLeft" },
  { key: "ArrowLeft" },
  { key: "ArrowLeft" },
  { text: "X" },
  { key: "z", mods: ["meta"] },
];
export default function Case() {
  const [html, setHtml] = useState("");
  const editor = useEditor({
    extensions: [StarterKit],
    content: "",
    immediatelyRender: true,
    editorProps: { attributes: { "data-testid": "pm", class: "h-32 w-80 border border-slate-400 p-2 text-sm outline-none [&_p]:my-1" } },
    onUpdate: ({ editor }) => setHtml(editor.getHTML()),
  });
  return (
    <div data-testid="root" className="h-full w-full bg-white p-4 text-slate-900">
      <EditorContent editor={editor} />
      <pre data-testid="doc" className="mt-2 w-96 whitespace-pre-wrap text-xs">
        {html}
      </pre>
    </div>
  );
}
