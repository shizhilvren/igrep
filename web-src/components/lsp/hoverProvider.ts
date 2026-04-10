import * as monaco from 'monaco-editor'
import { bisector } from 'd3-array'

type Position0 = {
    line: number
    character: number
}

type HoverLike = {
    start: Position0
    end: Position0
    hover: string
}

type SortedHoverItem = {
    startLine: number
    startChar: number
    endLine: number
    endChar: number
    hover: string
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

function buildSortedHoverItems(hovers: HoverLike[] | undefined): SortedHoverItem[] {
    if (!hovers || hovers.length === 0) {
        return []
    }

    return hovers
        .map((hover) => ({
            startLine: hover.start.line,
            startChar: hover.start.character,
            endLine: hover.end.line,
            endChar: hover.end.character,
            hover: hover.hover,
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

function toMonacoRange(item: SortedHoverItem): monaco.Range {
    return new monaco.Range(
        item.startLine + 1,
        item.startChar + 1,
        item.endLine + 1,
        item.endChar + 1,
    )
}

function containsPosition(item: SortedHoverItem, position: Position0Based): boolean {
    const startCmp = comparePosition(position.lineNumber, position.column, item.startLine, item.startChar)
    if (startCmp < 0) {
        return false
    }
    const endCmp = comparePosition(position.lineNumber, position.column, item.endLine, item.endChar)
    return endCmp < 0
}

const hoverStartBisector = bisector<SortedHoverItem, Position0Based>((item: SortedHoverItem, position: Position0Based) =>
    comparePosition(item.startLine, item.startChar, position.lineNumber, position.column),
)

function findHoverByBinarySearch(items: SortedHoverItem[], position: Position0Based): SortedHoverItem | undefined {
    if (items.length === 0) {
        return undefined
    }

    const candidate = hoverStartBisector.right(items, position) - 1

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

export function registerHoverProvider(language: string, hoverData: HoverLike[] | undefined): monaco.IDisposable {
    const sortedItems = buildSortedHoverItems(hoverData)

    return monaco.languages.registerHoverProvider(language, {
        provideHover(_, position) {
            const zeroBasedPosition = toZeroBasedPosition(position)
            const hover = findHoverByBinarySearch(sortedItems, zeroBasedPosition)
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
