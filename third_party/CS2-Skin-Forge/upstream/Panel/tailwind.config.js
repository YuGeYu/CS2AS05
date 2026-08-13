/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        cs2: {
          orange: '#de9b35',
          blue: '#5b9dd9',
          dark: '#1a1a2e',
          darker: '#0f0f1a',
          card: '#16213e',
          border: '#2a2a4a',
        },
      },
    },
  },
  plugins: [],
}
