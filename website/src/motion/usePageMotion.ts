import { useEffect } from "react";
import { gsap } from "gsap";

export function usePageMotion(key: string) {
  useEffect(() => {
    const media = gsap.matchMedia();
    media.add("(prefers-reduced-motion: no-preference)", () => {
      gsap.from("[data-intro]", {
        y: 16,
        duration: 0.7,
        stagger: 0.06,
        ease: "power3.out",
        clearProps: "transform",
      });
    });
    return () => media.revert();
  }, [key]);
}
