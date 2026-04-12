export class Files {
    files: Map<string, FileContent>

    constructor() {
        this.files = new Map()
    }
    public getFileContent(path: string[]) {
        let file_path = '/' + path.join('/')
        return this.files.get(file_path)
    }
    public addFileContent(path: string[], file: FileContent) {
        let file_path = '/' + path.join('/')
        this.files.set(file_path, file)
    }
}

export class FileContent {
    readonly filePath: string
    readonly code: string[]
    readonly language: string
    readonly semanticTokens: SemanticTokens
    hoverData?: HoverData[]
    definitionData?: DefinitionData[]

    constructor(filePath: string[], code: string[], language: string, semanticTokens: SemanticTokens) {
        this.code = code
        this.language = language
        this.semanticTokens = semanticTokens
        this.filePath = '/' + filePath.join('/')
    }
    public getHoverData(): HoverData[] | undefined {
        return this.hoverData
    }
    public setHoverData(val: HoverData[]) {
        this.hoverData = val
    }
    public getDefinitionData(): DefinitionData[] | undefined {
        return this.definitionData
    }
    public setDefinitionData(val: DefinitionData[]) {
        this.definitionData = val
    }
}

export class SemanticToken {
    readonly delta_line: number
    readonly delta_start: number
    readonly length: number
    readonly token_type: number
    readonly token_modifiers_bitset: number
    constructor(delta_line: number, delta_start: number, length: number, token_type: number, token_modifiers_bitset: number) {
        this.delta_line = delta_line
        this.delta_start = delta_start
        this.length = length
        this.token_type = token_type
        this.token_modifiers_bitset = token_modifiers_bitset
    }
}
export class SemanticTokens {
    readonly data: SemanticToken[]
    constructor(data: SemanticToken[]) {
        this.data = data
    }
}

export type HoverPosition = {
    line: number
    character: number
}

export class HoverData {
    readonly start: HoverPosition
    readonly end: HoverPosition
    readonly hover: string

    constructor(start: HoverPosition, end: HoverPosition, hover: string) {
        this.start = start
        this.end = end
        this.hover = hover
    }
}

export class DefinitionLocationModel {
    readonly fileName: string
    readonly start: HoverPosition
    readonly end: HoverPosition

    constructor(fileName: string, start: HoverPosition, end: HoverPosition) {
        this.fileName = fileName
        this.start = start
        this.end = end
    }
}

export class DefinitionData {
    readonly start: HoverPosition
    readonly end: HoverPosition
    readonly locations: DefinitionLocationModel[]

    constructor(start: HoverPosition, end: HoverPosition, locations: DefinitionLocationModel[]) {
        this.start = start
        this.end = end
        this.locations = locations
    }
}