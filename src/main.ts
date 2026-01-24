import { createApp } from 'vue';
import { createPinia } from 'pinia';
import './App.css';
import App from './App.vue';
import router from './router';
import { getApiKey, setApiKey, createGuestKey } from './api/server';

const app = createApp(App);
const pinia = createPinia();

app.use(pinia);
app.use(router);

// Initialize Auth for Web Mode
const isTauri = !!(window as any).__TAURI_INTERNALS__;
if (!isTauri) {
    const existingKey = getApiKey();
    if (!existingKey) {
        console.log('No API Key found for Web Mode. Registering as Guest...');
        // We can't await top-level comfortably in all bundlers, but we can try-catch
        // Or just let the first request fail? Better to init.
        createGuestKey().then(key => {
            console.log('Guest Key obtained:', key.api_key);
            setApiKey(key.api_key);
        }).catch(err => {
            console.error('Failed to obtain Guest Key:', err);
        });
    }
}

app.mount('#app');

