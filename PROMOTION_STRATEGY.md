# NoFuzz Tuner - Promotion & Feedback Strategy

Live at: https://www.nofuzz.app/

## Core Value Proposition

NoFuzz is a **free, privacy-first, browser-based guitar tuner** that runs entirely on the client using Rust compiled to WebAssembly. No account, no ads, no data collection, no install. It works offline after first load.

**Key differentiators to emphasize in all messaging:**
- Zero latency server round-trips - all processing happens locally via WASM
- Instrument-aware detection (acoustic, electric clean/distorted, bass, extended range, ukulele)
- Precise: YIN pitch detection with FFT refinement, harmonic correction, octave error correction
- Open source (MIT) - transparent, auditable, forkable
- Works on mobile and desktop browsers
- No signup, no ads, no tracking

---

## Audience Segments

### 1. Guitar Players (Primary - largest volume)
**Where they are:** Reddit (r/guitar, r/electricguitar, r/Bass, r/classicalguitar, r/ukulele), YouTube comments, guitar forums (The Gear Page, SevenString.org, TalkBass), Discord servers (guitar/music production communities), Facebook groups.

**What they care about:** Accuracy, speed, ease of use, free, works on phone.

**Messaging angle:** "A tuner that actually works well with distorted electric guitars and extended range instruments. No app install needed."

### 2. Web Developers / Rust Enthusiasts (Secondary - high amplification potential)
**Where they are:** Reddit (r/rust, r/webdev, r/WebAssembly), Hacker News, Dev.to, Twitter/X (#rustlang, #wasm), Lobste.rs, Mastodon.

**What they care about:** Interesting technical implementation, WASM use cases, performance.

**Messaging angle:** "Real-time audio processing in the browser using Rust + WebAssembly. Open source, MIT licensed. A practical showcase of what WASM can do."

### 3. Audio/DSP Engineers (Niche - credibility builders)
**Where they are:** DSP-related subreddits, audio programming Discord servers, KVR Audio forums.

**What they care about:** Algorithm quality, signal processing details.

**Messaging angle:** "YIN-based pitch detection with biquad filter banks, FFT refinement, and instrument-specific parameter tuning. All running in WASM."

---

## Phase 1: Foundation (Week 1-2)

### Improve the GitHub Repository
The repo is the landing page for technical audiences. Make it count.

- [ ] Add a screenshot/GIF of the tuner in action to the README (visual proof it works)
- [ ] Add a "Feedback" section to the README with a link to GitHub Issues
- [ ] Create issue templates: "Bug Report" and "Feature Request" with structured fields
- [ ] Add a CONTRIBUTING.md with setup instructions (reference CLAUDE.md content)
- [ ] Add GitHub Discussions to the repo (enable in repo settings) for open-ended feedback
- [ ] Pin a "Feedback Welcome" discussion thread

### Add In-App Feedback Mechanism
This is critical. Most users will never visit GitHub.

- [ ] Add a small "Feedback" button/link in the app UI that opens a short form or links to a simple feedback page
- [ ] Options for minimal friction:
  - A Google Form (3-4 questions: instrument type, what worked, what didn't, would you use again)
  - A link to GitHub Discussions (for technical users)
  - A mailto: link as absolute minimum fallback
- [ ] Add a subtle "Star us on GitHub" link for developer-audience visitors

### Set Up Analytics (Privacy-Respecting)
Understand usage without betraying the privacy-first promise.

- [ ] Consider privacy-respecting analytics: Plausible, Umami, or Fathom (all cookieless, GDPR-compliant)
- [ ] Track: page visits, instrument preset selection, tuning preset selection, session duration
- [ ] Do NOT track audio data or any personally identifiable information
- [ ] Mention the analytics tool transparently on the site if added

---

## Phase 2: Launch Posts (Week 2-3)

### Reddit Posts
Each subreddit has different norms. Tailor the post.

**r/guitar, r/electricguitar, r/Bass** (submit as text posts, not links):
- Title: "I built a free browser-based guitar tuner - looking for feedback"
- Lead with what makes it useful to THEM (instrument-specific detection, works on phone, no install)
- Ask specific questions: "Does it track accurately on your distorted tone?" / "How does it handle drop tunings for you?"
- Include link to https://www.nofuzz.app/
- Be transparent: you built it, you want honest feedback

**r/ukulele:**
- Tailor for ukulele specifically. Mention GCEA support.

**r/rust, r/WebAssembly:**
- Title: "Real-time pitch detection in the browser with Rust + WASM"
- Focus on technical architecture: YIN algorithm, biquad filters, audio worklets
- Link to both the app and the GitHub repo
- Ask for code review / technical feedback

**Hacker News (Show HN):**
- Title: "Show HN: NoFuzz - Browser-based guitar tuner built with Rust and WebAssembly"
- Write a concise comment explaining the technical stack and motivation
- HN audiences value technical depth and honest motivation

**Dev.to / Hashnode:**
- Write a short technical blog post: "Building a Real-Time Guitar Tuner with Rust, WebAssembly, and Svelte"
- Cover: why WASM for audio, YIN algorithm basics, challenges encountered, performance results
- Embed link to live demo

### Timing
- Post to different subreddits on different days (don't carpet-bomb)
- Best times for Reddit: Tuesday-Thursday, 9-11am EST
- HN: Tuesday-Thursday morning EST

---

## Phase 3: Sustained Visibility (Ongoing)

### Content Marketing
- [ ] Write 2-3 technical blog posts on Dev.to or a personal blog:
  1. "How YIN Pitch Detection Works (and how I implemented it in Rust)"
  2. "Why Electric Guitar Tuning Is Hard: Harmonics, Distortion, and Octave Errors"
  3. "Rust + WASM Performance: Processing Audio in Real-Time in the Browser"
- [ ] Cross-post or link from relevant Reddit/HN threads when the topic comes up organically
- [ ] Create a short (60-90 second) demo video showing the tuner in action with different instruments

### Community Engagement
- [ ] Monitor and respond to every piece of feedback (GitHub issues, discussions, Reddit comments)
- [ ] When users request features, create public GitHub issues to show progress
- [ ] Consider a changelog or "What's New" section on the site to show active development
- [ ] Engage in guitar/music subreddits naturally (not just self-promotion) to build credibility

### SEO / Discoverability
- [ ] Ensure the site has proper meta tags (title, description, Open Graph)
- [ ] Target keywords: "free online guitar tuner", "browser guitar tuner", "online bass tuner"
- [ ] Submit to web app directories: Product Hunt, AlternativeTo, ToolFinder
- [ ] Register on awesome-rust and awesome-wasm GitHub lists

---

## Feedback Collection Framework

### What to Measure

| Signal | Method | Purpose |
|--------|--------|---------|
| Accuracy perception | In-app feedback form | Core product quality |
| Instrument coverage | Form: "What instrument did you use?" | Prioritize presets |
| Comparison to alternatives | Form: "What tuner do you currently use?" | Competitive positioning |
| Pain points | Open text field | Feature prioritization |
| Return visits | Privacy-respecting analytics | Retention signal |
| GitHub stars | GitHub | Developer interest |
| Issue volume | GitHub Issues | Engagement depth |

### Key Questions for the Feedback Form
1. What instrument and tuning did you use?
2. How accurate did the tuner feel compared to what you normally use? (1-5 scale)
3. What device/browser are you using?
4. What would make you switch to this as your main tuner?
5. Anything that didn't work or felt off?

### Processing Feedback
- Review feedback weekly
- Categorize into: bugs, accuracy issues, UX improvements, feature requests
- Prioritize by frequency and impact
- Close the loop: when you fix something reported by a user, let them know

---

## Quick Wins (Do These First)

1. **Add a feedback link to the app** - even a simple mailto: link is better than nothing
2. **Take a screenshot/GIF** and add it to the GitHub README
3. **Write one Reddit post** in r/guitar asking for feedback
4. **Enable GitHub Discussions** on the repository
5. **Create issue templates** for bug reports and feature requests

These five actions can be done in a single session and immediately open up feedback channels.

---

## What NOT to Do

- Don't spam multiple subreddits on the same day
- Don't be defensive about negative feedback - it's the most valuable kind
- Don't add intrusive popups or modals asking for feedback in the app
- Don't add tracking that contradicts the privacy-first value proposition
- Don't wait for the product to be "perfect" before promoting - ship and iterate
