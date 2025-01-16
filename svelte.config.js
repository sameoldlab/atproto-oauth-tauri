import adNode from "@sveltejs/adapter-node";
import adStatic from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: process.env.MODE === 'native' ? adStatic() : adNode(),
  },
};

export default config;
