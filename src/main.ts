import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import "./assets/style.css";
import "prismjs/themes/prism-tomorrow.css";
import "prismjs/components/prism-json";
import "prismjs/components/prism-javascript";
import "prismjs/components/prism-bash";
import "prismjs/components/prism-markdown";

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.mount("#app");
