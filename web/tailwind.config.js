/** @type {import('tailwindcss').Config} */
module.exports = {
    content: [
        "./src/**/*.{rs,html,css}",
        "./assets/src/**/*.{html,css,js}",
        "./assets/dist/**/*.{html}"
    ],
    darkMode: "selector",
    theme: {
        extend: {
            borderImage: {
                'gold': 'linear-gradient(45deg, #462523 0%, #cb9b51 22%, #f6e27a 45%, #f6f2c0 50%, #f6e27a 55%, #cb9b51 78%, #462523 100%) 1'
            }
        }
    }
}
