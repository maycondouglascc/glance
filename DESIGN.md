# Glance: from noise to clarity

## Brief
Redesign the existing Linux process monitor landing page. Audience: Linux desktop users. Job: explain application grouping, let people try it, and install. Keep React/Vite, English/Portuguese, light/dark preference, actual project links and installation methods. Product facts come from README.md. Demo telemetry is explicitly illustrative, not a live benchmark. No invented credentials.

## Concepts considered
| Territory / idea | Signature / emotion / shareable frame | Anchor | Originality / fit / feasibility / portability |
| --- | --- | --- | --- |
| Name: see the whole system at a glance | Giant compact headline beside grouped applications; immediate clarity | Glance and progressive disclosure | 4 / 5 / 5 / 2 |
| Material: a process census | Dense process index resolves into application groups; precise control | cgroups and desktop entries | 4 / 5 / 5 / 2 |
| Process: the quiet observer | Resource numbers lead into a calm workspace; confidence | kernel-blocking idle waits | 3 / 4 / 5 / 3 |

Selected: combine the name's promise with the process census as its interactive proof. The page makes grouping tangible instead of decorating a generic download pitch.

## Concept card
Idea: Turn a noisy process list into a clear view of your applications.
Metaphor: A census of the desktop, organized by application.
Signature: Toggle an illustrative process list between raw processes and grouped applications; expand a group for detail.
Emotion: Relief and control in the first three seconds.
Motion verb: Gather and settle.
Screenshot: Oversized two-line headline opposite a compact process census, with a vermilion grouping signal.

## System
Palette: ground #f5f5f2; ink #20221f; muted #62665e; line #dadcd5; accent #b93818; curtain #20221f. Dark: ground #171916, ink #f0f1e9, muted #afb4a6, line #393d34, accent #ff896b. Accent comes from the warm signal in the existing Glance glyph, not a gradient identity.
Type: existing self-hosted Inter variable asset, consciously retained to keep brand continuity and avoid network fonts. Display uses tight -0.065em tracking, 0.98 leading and 64–112px fluid scale. Text 16–18px / 1.55. System monospace only for telemetry and commands.
Grid: 12-column equivalent, 48px outer gutter, hero 55/45 split, 4 columns mobile with 22px outer margin. Max 1440px. Alternate airy poster, compact numeric strip, asymmetric explanatory pair, installation workbench, footer finale.
Shape: 12px radius for app-like controls; pill only for primary download action.
Motion: power3.out, 240ms small actions, 650ms grouping. One GSAP ticker drives Lenis and signature transitions. No loader, no GL: this is a utility download page. Still at rest by design: the observer should not consume attention; meaningful grouping, hover, focus and press feedback supply the life. Reduced motion disables smooth scroll and all signature travel. Effects clean up on unmount and media preference change.

## Composition
```
Header: [glyph + Glance]                   [Explore / Install / GitHub] [EN PT / theme]
Hero:   [Linux monitor]                    [demo header / mode switch]
        [Less noise.]                     [process census]
        [More insight.]                   [expandable groups]
        [description / download]          [illustrative status]
Facts:  [0% idle CPU]        [~65 MB tray RSS]        [GTK4 + Rust]
Explore:[short heading]     [grouping / detail / safety in ruled rows]
Install:[large heading]     [method selector / command / copy feedback]
Footer: [giant Glance]                     [author / source / license]
```
Inner page: installation guide at ?page=guide, sharing header/footer and the installation workbench. Designed not-found state at ?page=404 and unknown paths. Query navigation keeps relative assets valid under the existing /glance deployment base.
Genericness test: removed centered hero, identical feature cards, gradient backdrop, badge collection and decorative counters. The process census is the main composition and interaction.

## Delivery
Production URL: https://www.maycondouglas.work/glance/. Source build remains website/dist; tracked docs is the existing publication directory and only gets updated when publishing is requested. No deployment in this task.

## Verification and review
Two production QA passes. Second comprehensive check: home, guide, 404 at 390/768/1440/1920; English and Portuguese, light/dark and reduced motion. Zero overflow, browser exceptions or axe violations. Grouping, group expansion and installer selection exercised. Final shoot: throttled mobile LCP 756ms, CLS 0, scroll 60FPS; desktop LCP600ms, CLS0. Keyboard: skip link first, no invisible or unstyled focus stops. Sampled primary hover controls all respond with motion; no noHoverChange or colorOnly findings. FPS is headless evidence, not a physical-device measurement.

Final static audit: zero FAILs. Three checker WARNs resolve URLs at the origin instead of /glance/: robots and sitemap exist at /glance/robots.txt and /glance/sitemap.xml; designed 404 with h1 is served with HTTP404 for missing URLs inside /glance/. The publication host must preserve 404 handling.

Separate critics: pass1 weighted7.21; pass2 weighted7.33 (Design7.5 / Usability7.4 / Creativity6.8 / Content7.8). The independent review did not reach the skill's aspirational8 threshold; do not represent it as an award-ready8. Principal opportunity: make the grouping transformation more distinctive across the composition. Addressed concrete feedback by adding the grouping proof, moving mobile demo forward, enlarging installer labels and providing sourced tray-mode metric context. Still page is intentional; no loader or WebGL.

Final accessory removed: decorative diagonal arrow above demo. Remaining assumptions: documented product facts and current README metrics, with explicit approximate memory note; sample process telemetry is illustrative. No fictional clients, awards or testimonials.
