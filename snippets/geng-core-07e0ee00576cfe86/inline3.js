
        export function setup(request, handler) {
            request.onreadystatechange = function () {
                if (request.readyState == 4) {
                    handler(request.status == 200);
                }
            };
        }
        