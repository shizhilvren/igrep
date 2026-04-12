import * as monaco from 'monaco-editor'
import type { HoverData } from './file'
import type { ComputedRef } from 'vue'

function toMonacoRange(item: HoverData): monaco.Range {
    return new monaco.Range(
        item.start.line + 1,
        item.start.character + 1,
        item.end.line + 1,
        item.end.character + 1,
    )
}

export function registerHoverProvider(language: string, hoverData: ComputedRef<Map<string, HoverData[] | undefined>>): monaco.IDisposable {

    return monaco.languages.registerHoverProvider(language, {
        provideHover(model, position) {
            const uri = model.uri
            const hovers = hoverData.value.get(uri.toString())
            const line = position.lineNumber - 1
            const char = position.column - 1
            const hover = hovers?.find((e) => {
                if (e.start.line == line && e.start.character <= char && char < e.end.character) {
                    return true
                } else {
                    return false
                }
            });
            if (!hover) {
                return null
            }

            return {
                range: toMonacoRange(hover),
                contents: [{ value: hover.hover }],
            }
        },
    })
}
