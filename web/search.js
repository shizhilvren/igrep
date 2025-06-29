import { ref } from 'vue'
export default {
    setup() {
        const count = ref(0)
        const msg = ref("")
        return { count, msg, greet }
    },
    template: `
<p>Message is: {{ msg }}</p>
<input v-model="msg" placeholder="edit me" class="form-control" type="email"/>
<button @click="greet(msg)" type="submit" class="btn btn-primary">Search</button>  
`
}

function greet(msg) {
    alert(`Hello ${msg}!`)

}