import path from 'node:path'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'
import { defineConfig } from 'vite'
export default defineConfig({plugins:[vue(),tailwindcss()],resolve:{alias:{'@':path.resolve(import.meta.dirname,'./src')}},server:{watch:{ignored:['**/src-tauri/target/**']}}})
