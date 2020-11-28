addEventListener('fetch', event => {
    event.respondWith(handleRequest(event.request));
});

async function handleRequest(request) {
    const { handle_request } = wasm_bindgen;
    await wasm_bindgen(wasm);
    return handle_request(request);
}
