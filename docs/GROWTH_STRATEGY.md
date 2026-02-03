# Growth Strategy for nofuzz.app

## Current State

- **Site**: https://www.nofuzz.app/
- **Analytics**: GoatCounter (jphollanti.goatcounter.com)
- **Type**: Browser-based guitar tuner (WebAssembly + Svelte)

## Traffic Acquisition Strategies

### 1. Search Engine Optimization (SEO)

#### Quick Wins
- [ ] **Add more long-tail keywords** to meta descriptions:
  - "free online guitar tuner"
  - "chromatic tuner no download"
  - "browser guitar tuner microphone"
  - "tune guitar online free"

- [ ] **Create instrument-specific landing pages**:
  - `/bass-tuner` - Bass guitar tuner
  - `/ukulele-tuner` - Ukulele tuner
  - `/acoustic-tuner` - Acoustic guitar tuner
  - Each page targets specific search queries

- [ ] **Add FAQ schema** for rich snippets:
  ```json
  {
    "@type": "FAQPage",
    "mainEntity": [
      {
        "@type": "Question",
        "name": "How accurate is this online guitar tuner?",
        "acceptedAnswer": {
          "@type": "Answer",
          "text": "Uses YIN pitch detection algorithm with FFT refinement for professional-grade accuracy within ±1 cent."
        }
      }
    ]
  }
  ```

- [ ] **Improve Core Web Vitals** - Already good with WASM, but ensure:
  - Fast LCP (Largest Contentful Paint)
  - No layout shifts (CLS)
  - Quick interactivity (FID/INP)

### 2. Content Marketing

#### Blog/Tutorial Ideas
- [ ] "How to Tune Your Guitar by Ear" (links to tuner)
- [ ] "Understanding Guitar Tuning: Standard vs Drop D"
- [ ] "Why Browser-Based Tuners Beat Mobile Apps"
- [ ] "Guide to Alternate Tunings for Guitar"

#### Video Content
- [ ] Short YouTube demo (under 60 seconds)
- [ ] TikTok/Instagram Reels showing the tuner in action
- [ ] Comparison video: nofuzz.app vs other tuners

### 3. Community Engagement

#### Reddit
- [ ] Post in r/guitar, r/guitarlessons, r/WeAreTheMusicMakers
- [ ] Respond to "looking for tuner" posts with genuine help
- [ ] r/WebDev for the technical implementation story

#### Forums & Communities
- [ ] Ultimate Guitar forums
- [ ] Gearslutz/Gearspace
- [ ] Acoustic Guitar Forum
- [ ] Discord guitar communities

#### Social Media
- [ ] Create @NoFuzzTuner accounts (Twitter handle exists per meta)
- [ ] Post tuning tips and tricks
- [ ] Engage with guitarist communities

### 4. Technical Improvements for Growth

#### PWA Enhancement
- [ ] Improve install prompts
- [ ] Add offline capability messaging
- [ ] Show "Add to Home Screen" prompt

#### Sharing Features
- [ ] Add "Share" button with Web Share API
- [ ] Social sharing cards are already good
- [ ] Add "Tuned with nofuzz.app" watermark option for screenshots

#### Referral/Viral Features
- [ ] "Tell a friend" prompt after successful tuning
- [ ] Integration with music lesson platforms

### 5. Partnerships & Backlinks

#### Music Education Sites
- [ ] Reach out to guitar lesson websites
- [ ] Music teacher directories
- [ ] School music programs

#### Music Gear Sites
- [ ] Guitar review sites
- [ ] Music equipment blogs
- [ ] Online music stores (as a free tool resource)

### 6. Paid Acquisition (if budget allows)

#### Google Ads
- Target keywords: "guitar tuner online", "free tuner"
- Low CPC in music tools niche

#### Music Platform Ads
- Spotify ads targeting guitarists
- YouTube pre-roll on guitar content

---

## Tracking & Analytics

### GoatCounter Setup

Your GoatCounter dashboard: https://jphollanti.goatcounter.com

### Check Analytics via CLI

```bash
# View dashboard (opens in browser)
./scripts/check-analytics.sh

# With API token for programmatic access
./scripts/check-analytics.sh --api YOUR_TOKEN
```

### Key Metrics to Track
1. **Daily/Weekly Unique Visitors**
2. **Referrer Sources** - Where traffic comes from
3. **Top Pages** - Which tunings are most popular
4. **Geographic Distribution**
5. **Device/Browser Stats** - Mobile vs Desktop

### Setting Up API Access

1. Log in to https://jphollanti.goatcounter.com
2. Navigate to Settings > API
3. Create a new API token
4. Save token to `~/.goatcounter_token` or set `GOATCOUNTER_API_TOKEN` env var

---

## Event Tracking Suggestions

Add custom GoatCounter events for deeper insights:

```javascript
// Track tuning completions
if (window.goatcounter) {
  window.goatcounter.count({
    path: '/events/tuning-complete',
    title: 'Tuning Complete',
    event: true
  });
}

// Track instrument selection
window.goatcounter.count({
  path: '/events/instrument/' + instrumentType,
  title: 'Instrument: ' + instrumentType,
  event: true
});

// Track tuning type selection
window.goatcounter.count({
  path: '/events/tuning/' + tuningName,
  title: 'Tuning: ' + tuningName,
  event: true
});
```

---

## Quick Action Checklist

### This Week
- [ ] Set up GoatCounter API token
- [ ] Post on r/guitar about the tuner
- [ ] Add FAQ schema to improve search snippets
- [ ] Create Twitter/X account @NoFuzzTuner

### This Month
- [ ] Create bass-tuner and ukulele-tuner landing pages
- [ ] Write one blog post about guitar tuning
- [ ] Add event tracking for tuning completions
- [ ] Reach out to 5 guitar lesson sites for backlinks

### This Quarter
- [ ] Create video content for YouTube/TikTok
- [ ] Build partnerships with music education sites
- [ ] Add more alternate tunings based on analytics
- [ ] Consider minimal Google Ads test campaign
