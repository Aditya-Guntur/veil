/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'veil': {
          'dark': '#000000',
          'primary': '#1a1a1a',
          'light': '#2a2a2a',
          'accent': '#ffffff',
          'cyan': '#e0e0e0',
          'purple': '#a0a0a0',
          'pink': '#808080',
        },
        'bg': {
          'dark': '#000000',
          'darker': '#0a0a0a',
          'card': '#111111',
        },
      },
      fontFamily: {
        'sans': ['Space Grotesk', 'Inter', 'sans-serif'],
        'mono': ['JetBrains Mono', 'monospace'],
      },
      animation: {
        'fade-in': 'fadeIn 0.3s ease-in',
      },
      keyframes: {
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
      },
      backgroundImage: {
        'gradient-radial': 'radial-gradient(var(--tw-gradient-stops))',
      },
    },
  },
  plugins: [],
}