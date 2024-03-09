/** @type {import('tailwindcss').Config} */

module.exports = {
    content: [
        "./app/**/*.js",
        "./components/**/*.js",
    ],
    theme: {
        extend: {},
    },
    plugins: [
        require('@tailwindcss/forms'),
    ],
}
