<svelte:head>
	<!-- Primary SEO -->
	<title>Guitar Tuner - Browser-Based &amp; Free - No-Fuzz</title>
	<meta name="description" content="Tune your guitar fast with browser-based tuner. No ads, 100% free." />
	<link rel="canonical" href="https://www.nofuzz.app/" />

	<!-- Open Graph -->
	<meta property="og:type"        content="website" />
	<meta property="og:title"       content="Guitar Tuner - Browser-Based &amp; Free - No-Fuzz" />
	<meta property="og:description" content="Tune your guitar accurately with browser-based tuner. No ads." />
	<meta property="og:url"         content="https://www.nofuzz.app/" />
	<meta property="og:image"       content="https://www.nofuzz.app/cover.png" />

	<!-- Twitter Card -->
	<meta name="twitter:card"        content="summary_large_image" />
	<meta name="twitter:site"        content="@NoFuzzTuner" />
	<meta name="twitter:title"       content="Guitar Tuner - Browser-Based &amp; Free - No-Fuzz" />
	<meta name="twitter:description" content="No-Fuzz guitar tuner." />
	<meta name="twitter:image"       content="https://www.nofuzz.app/cover.png" />

	<!-- PWA / install prompts -->
	<link rel="manifest" href="/manifest.webmanifest" />
	<meta name="theme-color" content="#121212" />

	<!-- App‑object schema for richer search results -->
	<script type="application/ld+json">
	{
	  "@context": "https://schema.org",
	  "@type": "SoftwareApplication",
	  "name": "No-Fuzz Guitar Tuner",
	  "applicationCategory": "MusicApplication",
	  "operatingSystem": "Web",
	  "offers": { "@type": "Offer", "price": "0", "priceCurrency": "USD" },
	  "featureList": [
	    "High-accuracy YIN pitch detection",
		"Standard tuning",
		"Drop D tuning",
	    "No ads or tracking"
	  ],
	  "screenshot": "https://www.nofuzz.app/cover.png",
	  "url": "https://www.nofuzz.app/"
	}
	</script>
</svelte:head>

<script>
	import '../app.css';
</script>

<div class="app">
	<main>
		<slot />
	</main>

	<footer>
		<p></p>
	</footer>
</div>

<style>
	:global(.app) {
		height: 100svh;                /* safe on load */
		display: flex;
		flex-direction: column;
	}
	@supports (height: 100dvh) {     /* modern browsers track URL-bar hide/show */
		:global(.app) { height: 100dvh; }
	}
	/* Chrome/Safari/Firefox modern override */
	@supports (height: 100dvh) {
		:global(.app) { min-height: 100dvh; }   /* tracks URL-bar hide/show */
	}

	main {
		flex: 1;
		flex: 1 0 auto;
		display: flex;
		flex-direction: column;
		box-sizing: border-box;
	}

	footer {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		padding: 12px;
	}

	@media (min-width: 480px) {
		footer {
			padding: 12px 0;
		}
	}
</style>
