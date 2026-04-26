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
            const line = position.lineNumber - 1
            const char = position.column - 1
            const wait = new Promise((resolve: (val: { contents: { value: string }, range: { start: { line: number, character: number }, end: { line: number, character: number } } } | null) => void) => {
                hover(uri.path, line, char, resolve as (val: unknown) => void)
            })
            const result = await wait;
            console.debug("hover id: ", result)
            if (result) {
                return {
                    range: {
                        startLineNumber: result.range.start.line + 1,
                        startColumn: result.range.start.character + 1,
                        endLineNumber: result.range.end.line + 1,
                        endColumn: result.range.end.character + 1
                    },
                    contents: [{
                        value: result.contents.value
                    }],
                }
            } else {
                return undefined
            }
        },
    })
}
