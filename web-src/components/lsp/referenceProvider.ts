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
    showNotification: (title: string, msg: string, type?: 'info' | 'success' | 'warning' | 'error', duration?: number) => { close: () => void } | undefined,
): monaco.IDisposable {
    return monaco.languages.registerReferenceProvider(language, {
        async provideReferences(model, position, _context, token) {
            const notified = showNotification('Finding references', 'Loading reference results...', "info", 0)
            try {
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

                const locations = reference?.locations.map((location) => ({
                    uri: monaco.Uri.parse('file://' + location.fileName),
                    range: toMonacoRange(location),
                })) ?? []

                showNotification('Success', 'Reference results loaded', 'success', 1500)
                return locations
            } catch (error) {
                console.error('Error providing references:', error)
                showNotification('Error', 'Failed to load reference results', 'error', 3000)
                return []
            } finally {
                notified?.close()
            }

        },
    })
}
