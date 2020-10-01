
        export function setup(image, handler) {
            image.onload = function() { handler(true); };
            image.onerror = function() { handler(false); };
        }
        