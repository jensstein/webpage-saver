/** @type {import('tailwindcss').Config} */

module.exports = {
    content: [
        "./app/**/*.js",
        "./components/**/*.js",
    ],
    theme: {
        extend: {
            // https://tailwindcss.com/docs/customizing-colors
            colors: {
                "sidebar": "#d1d5c8",
            }
        },
    },
    plugins: [
        require('@tailwindcss/forms'),
    ],
}
