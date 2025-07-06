import { ref, h, computed, defineComponent, type Plugin, watch } from 'vue'
import hljs from 'highlight.js/lib/core'
import { escapeHtml } from './utils'
import type { BeforeHighlightContext, HighlightResult } from 'highlight.js';
import type { FileDataMatchRange } from '../../pkg/igrep';
import { progressProps } from 'element-plus';


const component = defineComponent({
    props: {
        code: {
            type: String,
            required: true,
        },
        ranges: {
            type: Array as () => Array<FileDataMatchRange>,
            default: () => [],
        },
        language: {
            type: String,
            default: '',
        },
        autodetect: {
            type: Boolean,
            default: true,
        },
        ignoreIllegals: {
            type: Boolean,
            default: true,
        },
    },

    setup(props) {

        const language = ref(props.language)
        watch(() => props.language, (newLanguage) => {
            language.value = newLanguage
        })

        const autodetect = computed(() => props.autodetect && !language.value)
        const cannotDetectLanguage = computed(() => !autodetect.value && !hljs.getLanguage(language.value))

        const className = computed((): string => {
            if (cannotDetectLanguage.value) {
                return ''
            } else {
                return `hljs ${language.value}`
            }
        })

        const highlightedCode = computed((): string => {
            // No idea what language to use, return raw code
            if (cannotDetectLanguage.value) {
                console.warn(`The language "${language.value}" you specified could not be found.`)
                return escapeHtml(props.code)
            }

            if (autodetect.value) {
                const result = hljs.highlightAuto(props.code)
                language.value = result.language ?? ''
                return result.value
            } else {
                const result = hljs.highlight(props.code, {
                    language: language.value,
                    ignoreIllegals: props.ignoreIllegals,
                })
                return result.value
            }
        })
        const fun = function fun(text: string) {
            const tempContainer = document.createElement('div');
            tempContainer.innerHTML = text;
            // console.log(text)
            let rangs_set = new Set<number>()
            for (let item of props.ranges) {
                for (let i = item.start; i < item.end; i++) {
                    rangs_set.add(i)
                }
            }
            const ans = traverse(tempContainer, rangs_set, 0)
            // console.log(ans.result)
            return ans.result;
        }
        return {
            className,
            highlightedCode,
            fun,
        }
    },
    render() {
        let ans = this.fun(this.highlightedCode)
        let a = h('pre', {}, [
            h('code', {
                class: this.className,
                innerHTML: ans,
                tabindex: '0',
                ref: "code"
            }),
        ])
        return a
    },
})

const plugin: Plugin & { component: typeof component } = {
    install(app) {
        app.component('highlightjs', component)
    },
    component,
}

export default plugin

const traverse = (node: Element | ChildNode, ranges: Set<number>, id: number) => {
    let result = ""
    const indent = ' ';
    if (node.nodeType === Node.ELEMENT_NODE && node instanceof HTMLElement) {
        // 处理元素节点
        result += `<${node.tagName.toLowerCase()}`;
        for (let att of node.attributes) {
            result += ` ${att.name}="${att.value}"`;
        }
        result += `>`;

        // 递归子节点
        for (let child of node.childNodes) {
            let { result: text, id: len } = traverse(child, ranges, id);
            result += text;
            id = len;
        }
        result += `</${node.tagName.toLowerCase()}>`;
    } else if (node.nodeType === Node.TEXT_NODE) {
        // 处理文本节点
        const text = node.textContent;
        if (text !== null) {
            let match_text = ""
            for (let i = 0; i < text.length; i++) {
                if (ranges.has(id + i)) {
                    match_text += text[i];
                    // result += `<span class="match-highlight">${text[i]}</span>`;
                } else {
                    if (match_text.length > 0) {
                        result += `<span class="match-highlight">${match_text}</span>`;
                        match_text = "";
                    }
                    result += text[i];
                }
            }
            if (match_text.length > 0) {
                result += `<span class="match-highlight">${match_text}</span>`;
                match_text = "";
            }
            // console.log(text)
            id += text.length
        }
    }
    return { result: result, id: id };
};