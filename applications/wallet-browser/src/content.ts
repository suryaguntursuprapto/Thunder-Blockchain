// Inject the inject.ts script into the main DOM
const script = document.createElement('script');
script.src = chrome.runtime.getURL('assets/inject.js');
(document.head || document.documentElement).appendChild(script);

// Relay messages from the injected script to the background service worker
window.addEventListener('message', (event) => {
  if (event.source !== window || !event.data || event.data.type !== 'THUNDER_REQUEST') {
    return;
  }

  // Send to background script
  chrome.runtime.sendMessage(
    { method: event.data.method, params: event.data.params },
    (response: any) => {
      // Send the response back to the injected script
      window.postMessage(
        {
          type: 'THUNDER_RESPONSE',
          id: event.data.id,
          result: response
        },
        '*'
      );
    }
  );
});
