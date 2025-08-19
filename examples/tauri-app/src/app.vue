<script setup lang="ts">
import { ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { signIn, getUser } from 'tauri-plugin-authium-api';

const response = ref('');
const user = ref<any>(null);

getUser().then((userData) => {
    user.value = userData;
    updateResponse(`User data: ${JSON.stringify(user.value)}`);
}).catch((error) => {
    updateResponse(`Error fetching user data: ${error.message}`);
});

listen("authium:login-success", (event) => {
    user.value = event.payload;
    updateResponse(`Login successful: ${JSON.stringify(user.value)}`);
});

function updateResponse(returnValue) {
    response.value += `[${new Date().toLocaleTimeString()}] `
        + (typeof returnValue === 'string' ? returnValue : JSON.stringify(returnValue))
        + '<br>';
}
</script>

<template>
    <main class="container">
        <h1>Welcome to Tauri!</h1>

        <div class="row">
            <a href="https://vite.dev" target="_blank">
                <img src="/vite.svg" class="logo vite" alt="Vite Logo" />
            </a>
            <a href="https://tauri.app" target="_blank">
                <img src="/tauri.svg" class="logo tauri" alt="Tauri Logo" />
            </a>
            <a href="https://vuejs.org" target="_blank">
                <img src="/svelte.svg" class="logo vue" alt="Vue Logo" />
            </a>
        </div>

        <p>
            Click on the Tauri, Vite, and Vue logos to learn more.
        </p>

        <div>
            <button @click="() => signIn()">{{ user ? "You are logged in" : "Sign In" }}</button>
            <div v-html="response"></div>
        </div>
    </main>
</template>

<style>
.logo.vite:hover {
  filter: drop-shadow(0 0 2em #747bff);
}

.logo.vue:hover {
  filter: drop-shadow(0 0 2em #42b883);
}
</style>
