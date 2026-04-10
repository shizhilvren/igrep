import * as monaco from 'monaco-editor'
import { bisector } from 'd3-array'
import { PathIndex, TreeData } from 'igrep'
import { fetchFileData } from '@/utils/utils'

type Position0 = {
    line: number
    character: number
}

type DefinitionLocationLike = {
    fileName: string
    start: Position0
    end: Position0
}

type DefinitionLike = {
    start: Position0
    end: Position0
    locations: DefinitionLocationLike[]
}

type SortedRangeItem = {
    startLine: number
    startChar: number
    endLine: number
    endChar: number
}

type SortedDefinitionLocation = SortedRangeItem & {
    fileName: string
}

type SortedDefinitionItem = SortedRangeItem & {
    locations: SortedDefinitionLocation[]
}

type Position0Based = {
    lineNumber: number
    column: number
}

function comparePosition(aLine: number, aChar: number, bLine: number, bChar: number): number {
    if (aLine !== bLine) {
        return aLine - bLine
    }
    return aChar - bChar
}

function buildSortedDefinitionItems(definitions: DefinitionLike[] | undefined): SortedDefinitionItem[] {
    if (!definitions || definitions.length === 0) {
        return []
    }

    return definitions
        .map((definition) => ({
            startLine: definition.start.line,
            startChar: definition.start.character,
            endLine: definition.end.line,
            endChar: definition.end.character,
            locations: definition.locations.map((location) => ({
                fileName: location.fileName,
                startLine: location.start.line,
                startChar: location.start.character,
                endLine: location.end.line,
                endChar: location.end.character,
            })),
        }))
        .sort((a, b) => {
            const startCmp = comparePosition(a.startLine, a.startChar, b.startLine, b.startChar)
            if (startCmp !== 0) {
                return startCmp
            }
            return comparePosition(a.endLine, a.endChar, b.endLine, b.endChar)
        })
}

function toZeroBasedPosition(position: monaco.Position): Position0Based {
    return {
        lineNumber: position.lineNumber - 1,
        column: position.column - 1,
    }
}

function toMonacoRange(item: SortedRangeItem): monaco.Range {
    return new monaco.Range(
        item.startLine + 1,
        item.startChar + 1,
        item.endLine + 1,
        item.endChar + 1,
    )
}

function containsPosition(item: SortedRangeItem, position: Position0Based): boolean {
    const startCmp = comparePosition(position.lineNumber, position.column, item.startLine, item.startChar)
    if (startCmp < 0) {
        return false
    }
    const endCmp = comparePosition(position.lineNumber, position.column, item.endLine, item.endChar)
    return endCmp < 0
}

const definitionStartBisector = bisector<SortedDefinitionItem, Position0Based>((item: SortedDefinitionItem, position: Position0Based) =>
    comparePosition(item.startLine, item.startChar, position.lineNumber, position.column),
)

function findDefinitionByBinarySearch(items: SortedDefinitionItem[], position: Position0Based): SortedDefinitionItem | undefined {
    if (items.length === 0) {
        return undefined
    }

    const candidate = definitionStartBisector.right(items, position) - 1

    if (candidate === -1) {
        return undefined
    }

    const current = items[candidate]
    if (current && containsPosition(current, position)) {
        return current
    }

    if (candidate > 0) {
        const previous = items[candidate - 1]
        if (previous && containsPosition(previous, position)) {
            return previous
        }
    }

    return undefined
}

function locationPathToUri(path: string): monaco.Uri {
    const normalized = path
        .split('/')
        .filter((segment) => segment.length > 0)
        .join('/')
    return monaco.Uri.parse(`file:///${normalized}`)
}

async function ensureModelForLocation(filePath: string, language: string, token: monaco.CancellationToken): Promise<void> {
    if (token.isCancellationRequested) {
        return
    }

    const uri = locationPathToUri(filePath)
    if (monaco.editor.getModel(uri)) {
        return
    }

    let pathIndex = new PathIndex(filePath)
    let a = pathIndex.path_str('lsp-index');
    console.log('LSP index path for file', filePath, ':', a)
    const data = await fetchFileData(a + '/tree.data');

    if (!data || token.isCancellationRequested) {
        return
    }

    const treeData = new TreeData(data)
    if (!treeData.is_file()) {
        return
    }

    const fileData = treeData.file_data()
    if (!fileData) {
        return
    }

    const modelText = fileData.lines().join('\n')

    if (!monaco.editor.getModel(uri)) {
        monaco.editor.createModel(modelText, language, uri)
    }
}

export function registerDefinitionProvider(language: string, definitionData: DefinitionLike[] | undefined): monaco.IDisposable {
    const sortedItems = buildSortedDefinitionItems(definitionData)

    return monaco.languages.registerDefinitionProvider(language, {
        async provideDefinition(_, position, token) {
            const zeroBasedPosition = toZeroBasedPosition(position)
            const definition = findDefinitionByBinarySearch(sortedItems, zeroBasedPosition)
            if (!definition || definition.locations.length === 0) {
                return null
            }

            const uniquePaths = [...new Set(definition.locations.map((location) => location.fileName))]
            await Promise.all(uniquePaths.map((filePath) => ensureModelForLocation(filePath, language, token)))

            if (token.isCancellationRequested) {
                return null
            }

            return definition.locations.map((location) => ({
                uri: locationPathToUri(location.fileName),
                range: toMonacoRange(location),
            }))
        },
    })
}
