/**
 * GoatCounter Analytics Utilities
 *
 * Track custom events to understand user behavior better.
 * All tracking is privacy-respecting (GoatCounter doesn't track personal data).
 */

declare global {
    interface Window {
        goatcounter?: {
            count: (options: {
                path: string;
                title?: string;
                referrer?: string;
                event?: boolean;
            }) => void;
        };
    }
}

/**
 * Track a custom event in GoatCounter
 */
export function trackEvent(eventName: string, eventTitle?: string): void {
    if (typeof window !== 'undefined' && window.goatcounter) {
        window.goatcounter.count({
            path: `/events/${eventName}`,
            title: eventTitle ?? eventName,
            event: true
        });
    }
}

/**
 * Track instrument selection
 */
export function trackInstrumentSelection(instrument: string): void {
    trackEvent(`instrument/${instrument}`, `Instrument: ${instrument}`);
}

/**
 * Track tuning selection
 */
export function trackTuningSelection(tuningId: string): void {
    trackEvent(`tuning/${tuningId}`, `Tuning: ${tuningId}`);
}

/**
 * Track when audio is successfully started (microphone permission granted)
 */
export function trackAudioStart(): void {
    trackEvent('audio-start', 'Audio Started');
}

/**
 * Track successful tuning (user stayed on a note within ±2 cents for a while)
 */
export function trackTuningSuccess(note: string): void {
    trackEvent(`tuning-success/${note}`, `Tuned: ${note}`);
}

/**
 * Track session duration (call when user leaves)
 */
export function trackSessionDuration(durationSeconds: number): void {
    const bucket =
        durationSeconds < 30 ? 'under-30s' :
        durationSeconds < 60 ? '30s-1min' :
        durationSeconds < 300 ? '1-5min' :
        durationSeconds < 600 ? '5-10min' : 'over-10min';
    trackEvent(`session-duration/${bucket}`, `Session: ${bucket}`);
}
