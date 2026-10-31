import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import { detectLocale, i18n, setLocale } from "./i18n";
import "./style.css";

// The loading screen already needs the texts of the system language.
await setLocale(detectLocale("auto"));
createApp(App).use(createPinia()).use(i18n).mount("#app");
