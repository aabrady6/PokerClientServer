<template>
  <div class="login_container">
    <h1>Balatro</h1>
    <h2>Login or Register</h2>
    <input
      v-model="username"
      type="text"
      placeholder="Enter your username..."
      @keyup.enter="login"
      class="input-field mb-4"
    />
    <input
      v-model="password"
      type="password"
      placeholder="Enter your password..."
      @keyup.enter="login"
      class="input-field mb-4"
    />
    <div class="button-container">
      <button @click="login" class="control_button">Login</button>
      <button @click="register" class="control_button">Register</button>
    </div>
    <p v-if="errorMessage" class="error mt-4 text-red-500">
      {{ errorMessage }}
    </p>
  </div>
</template>
<script setup>
import { ref } from "vue";

const NODE_SERVER_IP = import.meta.env.NODE_SERVER_IP || "localhost";
const NODE_SERVER_PORT = import.meta.env.NODE_SERVER_PORT || "3000";

const username = ref("");
const password = ref("");
const errorMessage = ref("");

const emit = defineEmits(["loggedIn"]);

async function login() {
  if (!username.value.trim()) {
    errorMessage.value = "Please enter a valid username.";
    return;
  }

  if (!password.value.trim()) {
    errorMessage.value = "Please enter a valid password.";
    return;
  }

  try {
    const response = await fetch(
      `http://${NODE_SERVER_IP}:${NODE_SERVER_PORT}/login/${username.value}`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          username: username.value,
          password: password.value,
        }),
      }
    );

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error);
    }

    const data = await response.json();
    console.log("Login successful:", data);

    emit("loggedIn", {
      username: data.username,
    });
  } catch (error) {
    console.error("Login error:", error);
    errorMessage.value = "Incorrect Credentials.";
  }
}

async function register() {
  if (!username.value.trim()) {
    errorMessage.value = "Please enter a valid username.";
    return;
  }

  if (!password.value.trim()) {
    errorMessage.value = "Please enter a valid password.";
    return;
  }

  try {
    const response = await fetch(
      `http://${NODE_SERVER_IP}:${NODE_SERVER_PORT}/register/${username.value}`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          username: username.value,
          password: password.value,
        }),
      }
    );

    if (!response.ok) {
      const error = await response.text();
      throw new Error(error);
    }

    const data = await response.json();
    console.log("Registration successful:", data);

    emit("loggedIn", {
      username: data.username,
    });
  } catch (error) {
    console.error("Registration error:", error);
    errorMessage.value = "User Already Exists";
  }
}
</script>

<style scoped lang="postcss">
@reference "tailwindcss";

.login_container {
  @apply flex flex-col justify-center items-center m-8 h-full;

  h1 {
    @apply text-6xl font-bold mb-6;
  }

  h2 {
    @apply text-3xl font-bold mb-6;
  }

  .input-field {
    @apply w-full p-2 border border-black mb-4 rounded-md text-base focus:outline-none focus:ring-2 focus:ring-green-500;
  }

  .button-container {
    @apply flex flex-row gap-4 text-base;
  }

  .control_button {
    @apply border border-black bg-white p-2 select-none;

    &:hover {
      @apply bg-gray-400 cursor-pointer;
    }

    &:active {
      @apply bg-green-500 cursor-pointer;
    }

    &.unavailable {
      @apply cursor-not-allowed bg-gray-300 brightness-75;
    }

    &.selected {
      @apply bg-green-500;
    }
  }

  .error {
    @apply text-red-500;
  }
}
</style>
