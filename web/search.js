import { ref } from 'vue'
// import init, { greet } from "./pkg/igrep.js";


export default {
    setup() {
        const count = ref(0)
        const msg = ref("")
        // rust_api_init()
        console.log("rust load finish")
        return { count, msg, greet_2 }
    },
    mounted() {
        console.log(`the component is now mounted.`)
    },
    template: `
<p>Message is: {{ msg }}</p>
<input v-model="msg" placeholder="edit me" class="form-control" type="email"/>
<button @click="greet_2(msg)" type="submit" class="btn btn-primary">Search</button>  
`
}


function greet_2(msg) {
    // alert(`Hello ${msg}!`)
    // greet(msg);
}

function rust_api_init() {
    init().then(() => {
        greet_2("efgh");
    })
}