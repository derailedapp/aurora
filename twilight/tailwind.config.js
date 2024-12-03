/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./app/**/*.{js,jsx,ts,tsx}", "./components/**/*.{js,jsx,ts,tsx}"],
  presets: [require("nativewind/preset")],
  theme: {
    extend: {
      fontFamily: {
        main: '"Reddit Sans", sans-serif',
      },
      colors: {
        brand: "rgb(115 100 255)",
        "not-quite-dark-blue": "rgb(10 22 40)",
      },
    },
  },
  plugins: [],
};
