import * as monaco from 'monaco-editor'
import type { SemanticTokens } from '@/components/lsp/OneFile.vue'

const tokenTypeClassMap = [
    'st-file',
    'st-module',
    'st-namespace',
    'st-package',
    'st-class',
    'st-method',
    'st-property',
    'st-field',
    'st-constructor',
    'st-enum',
    'st-interface',
    'st-function',
    'st-variable',
    'st-constant',
    'st-string',
    'st-number',
    'st-boolean',
    'st-array',
    'st-object',
    'st-key',
    'st-null',
    'st-enum-member',
    'st-struct',
    'st-event',
    'st-operator',
    'st-type-parameter',

    // Keep semantic-token-specific values for compatibility with existing back-end indexes.
    'st-type',
    'st-parameter',
    'st-macro',
    'st-keyword',
    'st-modifier',
    'st-comment',
    'st-regexp',
    'st-decorator',
]

export function applySemanticHighlight(
    editor: monaco.editor.IStandaloneCodeEditor | null,
    semanticTokens: SemanticTokens | undefined,
    decorations: monaco.editor.IEditorDecorationsCollection | null,
): monaco.editor.IEditorDecorationsCollection | null {
    if (!editor) {
        return decorations
    }

    const model = editor.getModel()
    if (!model) {
        return decorations
    }

    const tokens = semanticTokens ?? { data: [] }
    if (tokens.data.length === 0) {
        decorations?.set([])
        return decorations
    }

    const next: monaco.editor.IModelDeltaDecoration[] = []
    let line = 0
    let start = 0

    for (const token of tokens.data) {
        line += token.delta_line
        if (token.delta_line === 0) {
            start += token.delta_start
        } else {
            start = token.delta_start
        }

        const lineNumber = line + 1
        if (lineNumber < 1 || lineNumber > model.getLineCount()) {
            continue
        }

        const startColumn = start + 1
        const endColumn = startColumn + token.length
        const lineMaxColumn = model.getLineMaxColumn(lineNumber)
        const safeStart = Math.max(1, Math.min(startColumn, lineMaxColumn))
        const safeEnd = Math.max(safeStart, Math.min(endColumn, lineMaxColumn))
        const typeClass = tokenTypeClassMap[token.token_type] ?? 'st-default'

        next.push({
            range: new monaco.Range(lineNumber, safeStart, lineNumber, safeEnd),
            options: { inlineClassName: `semantic-token ${typeClass}` },
        })
    }

    if (!decorations) {
        return editor.createDecorationsCollection(next)
    }

    decorations.set(next)
    return decorations
}
