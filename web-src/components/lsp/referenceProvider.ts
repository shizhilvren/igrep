import * as monaco from 'monaco-editor'
import type { ComputedRef } from 'vue'
import type { ReferenceData, ReferenceLocationModel } from './file'

function toMonacoRange(item: ReferenceLocationModel): monaco.Range {
    return new monaco.Range(
        item.start.line + 1,
        item.start.character + 1,
        item.end.line + 1,
        item.end.character + 1,
    )
}

export function registerReferenceProvider(
    language: string,
    referenceData: ComputedRef<Map<string, ReferenceData[] | undefined>>,
    addFileToModel: (pathName: string) => Promise<void> | null,
): monaco.IDisposable {
    return monaco.languages.registerReferenceProvider(language, {
        async provideReferences(model, position, _context, token) {
            const uri = model.uri
            const references = referenceData.value.get(uri.toString())
            const line = position.lineNumber - 1
            const char = position.column - 1
            const reference = references?.find((e) => {
                if (e.start.line == line && e.start.character <= char && char < e.end.character) {
                    return true
                } else {
                    return false
                }
            })

            const uniquePaths = [...new Set(reference?.locations.map((location) => location.fileName))]
                .map((filePath) => addFileToModel(filePath))
                .filter((e) => !!e)
            await Promise.all(uniquePaths)

            if (token.isCancellationRequested) {
                return null
            }

            return reference?.locations.map((location) => ({
                uri: monaco.Uri.parse('file://' + location.fileName),
                range: toMonacoRange(location),
            })) ?? []
        },
    })
}
