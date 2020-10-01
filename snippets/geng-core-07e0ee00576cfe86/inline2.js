
        export function setup(audio, handler) {
            audio.oncanplaythrough = function() { handler(true); };
            audio.onerror = function() { handler(false); };
        }
        