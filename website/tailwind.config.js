/** @type {import('tailwindcss').Config} */
const zincShades = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950]
const paletteAwareZinc = Object.fromEntries(
  zincShades.map((shade) => [
    shade,
    "hsl(var(--palette-neutral-hue, 260) var(--palette-neutral-saturation, 4%) var(--palette-zinc-" +
      shade +
      ") / <alpha-value>)",
  ]),
)

export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        zinc: paletteAwareZinc,
        accent: {
          DEFAULT:
            "hsl(var(--palette-accent-hue, 258) var(--palette-accent-saturation, 48%) var(--palette-accent-lightness, 40%) / <alpha-value>)",
          hover:
            "hsl(var(--palette-accent-hue, 258) var(--palette-accent-saturation, 48%) var(--palette-accent-hover-lightness, 30%) / <alpha-value>)",
        },
      },
      boxShadow: {
        xs: "0px 1px 2px 0px rgba(0, 0, 0, 0.05)",
        sm: "0px 1px 2px 0px rgba(0, 0, 0, 0.06), 0px 1px 3px 0px rgba(0, 0, 0, 0.10)",
        md: "0px 2px 4px -1px rgba(0, 0, 0, 0.06), 0px 4px 6px -1px rgba(0, 0, 0, 0.08)",
        lg: "0px 4px 6px -2px rgba(0, 0, 0, 0.05), 0px 10px 15px -3px rgba(0, 0, 0, 0.08)",
        xl: "0px 10px 10px -5px rgba(0, 0, 0, 0.04), 0px 20px 25px -5px rgba(0, 0, 0, 0.10)",
        "2xl": "0px 25px 50px -12px rgba(0, 0, 0, 0.25)",
        inner: "inset 0px 2px 4px 0px rgba(0, 0, 0, 0.06)",
      },
      fontFamily: {
        sans: ["Inter", "-apple-system", "BlinkMacSystemFont", "sans-serif"],
        mono: ["JetBrains Mono", "ui-monospace", "SFMono-Regular", "monospace"],
      },
      keyframes: {
        fadeIn: {
          "0%": { opacity: "0", transform: "translateY(8px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
      },
      animation: {
        "fade-in": "fadeIn 0.3s ease-in forwards",
      },
      fontSize: {
        "display-64-medium": ["64px", { lineHeight: "70px", letterSpacing: "-0.04em" }],
        "display-56-medium": ["56px", { lineHeight: "62px", letterSpacing: "-0.035em" }],
        "heading-40-medium": ["40px", { lineHeight: "48px", letterSpacing: "-0.025em" }],
        "heading-32-medium": ["32px", { lineHeight: "40px", letterSpacing: "-0.015em" }],
        "heading-28-medium": ["28px", { lineHeight: "36px", letterSpacing: "-0.01em" }],
        "subheading-24-medium": ["24px", { lineHeight: "32px", letterSpacing: "-0.0075em" }],
        "subheading-20-medium": ["20px", { lineHeight: "28px", letterSpacing: "-0.0025em" }],
        "body-18-medium": ["18px", { lineHeight: "27px", letterSpacing: "0em" }],
        "body-18-regular": ["18px", { lineHeight: "28px", letterSpacing: "0em" }],
        "body-16-medium": ["16px", { lineHeight: "25px", letterSpacing: "0em" }],
        "body-16-regular": ["16px", { lineHeight: "26px", letterSpacing: "0em" }],
        "body-15-medium": ["15px", { lineHeight: "23px", letterSpacing: "0em" }],
        "body-15-regular": ["15px", { lineHeight: "24px", letterSpacing: "0em" }],
        "body-14-medium": ["14px", { lineHeight: "22px", letterSpacing: "0.0025em" }],
        "body-14-regular": ["14px", { lineHeight: "22px", letterSpacing: "0em" }],
        "caption-13-medium": ["13px", { lineHeight: "21px", letterSpacing: "0.0075em" }],
        "caption-13-regular": ["13px", { lineHeight: "21px", letterSpacing: "0.005em" }],
        "caption-12-medium": ["12px", { lineHeight: "19px", letterSpacing: "0.0125em" }],
        "caption-12-regular": ["12px", { lineHeight: "20px", letterSpacing: "0.01em" }],
        "caption-11-regular": ["11px", { lineHeight: "18px", letterSpacing: "0.015em" }],
      },
    },
  },
  plugins: [],
}
