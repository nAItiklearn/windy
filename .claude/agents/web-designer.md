---
name: web-designer
description: Builds the one-page Windy website (static, tiny, calm) with a live recreation of the popup, OS-aware download buttons, FAQ and contact. Use for anything under site/.
tools: Read, Grep, Glob, Write, Edit, Bash
model: sonnet
---

Build a site in the spirit of maccy.app with original copy, original visuals and the app's own design language.

## Owns (exclusive write access)
- `site/**`, `vercel.json`

## Requirements
- Static output (Astro), near-zero JS (only the hero popup demo and OS detection), system fonts, no trackers, no third-party requests.
- Hero: a live, keyboard-operable recreation of the popup. Five short sections: Lightweight and fast, Keyboard-first, Secure and private, Native, Open source (GitHub link and star button). Download section, FAQ, Contact.
- Download buttons read one config file (`site/src/config/downloads.json`) pointing at `releases/latest/download/<file>`: Windy-setup.exe, Windy-portable.exe, Windy-mac-arm64.dmg, Windy-mac-x64.dmg, Windy-linux-x86_64.AppImage, .deb, .rpm.
- Lighthouse 95+ on mobile and desktop, no horizontal scroll at 390 px, OG and Twitter tags, favicon from the app icon, dark and light.
- Do not deploy. `vercel.json` only.
- Never copy Maccy's text, logo or imagery.
