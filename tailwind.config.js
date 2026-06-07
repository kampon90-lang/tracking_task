/** @type {import('tailwindcss').Config} */
module.exports = {
  mode: "all",
  content: [
    // บอกให้ Tailwind เข้าไปสแกนหาคลาสในไฟล์ .rs ของเรา
    "./src/**/*.{rs,html,css}",
  ],
  theme: {
    extend: {},
  },
  plugins: [],
};
