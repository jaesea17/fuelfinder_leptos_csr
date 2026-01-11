/** @type {import('tailwindcss').Config} */
export default {
  darkMode: "class",
  content: ["./src/**/*.rs", "./index.html"], // Simplified array
  theme: {
    extend: {},
  },
  plugins: [],
};

// npx @tailwindcss/cli -i ./style/tailwind.css -o ./style/output.css --watch
