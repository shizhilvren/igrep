import * as monaco from 'monaco-editor'
import type { HoverData } from './file'
import type { ComputedRef, Ref } from 'vue'

function toMonacoRange(item: HoverData): monaco.Range {
    return new monaco.Range(
        item.start.line + 1,
        item.start.character + 1,
        item.end.line + 1,
        item.end.character + 1,
    )
}

export function registerHoverProvider(language: string, hover: (file_path: string, line: number, character: number, resolve: (any: unknown) => void) => void): monaco.IDisposable {

    return monaco.languages.registerHoverProvider(language, {
        async provideHover(model, position) {
            const uri = model.uri
            const line = position.lineNumber
            const char = position.column
            const wait = new Promise((resolve) => {
                hover(uri.fsPath, line, char, resolve)
            })
            const id = await wait;
            console.debug("hover id: ", id)
            return {
                range: undefined,
                contents: [{ value: "hover.hover " }],
            }
        },
    })
}
