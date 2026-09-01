// Injected into the page's main DOM context

class ThunderProvider {
  isThunder = true;

  async request(args: { method: string, params?: any[] }) {
    return new Promise((resolve, reject) => {
      // Create a unique ID for this request
      const id = Date.now().toString() + Math.random().toString();
      
      const listener = (event: MessageEvent) => {
        if (event.source !== window || !event.data || event.data.type !== 'THUNDER_RESPONSE' || event.data.id !== id) {
          return;
        }
        
        window.removeEventListener('message', listener);
        
        if (event.data.error) {
          reject(new Error(event.data.error));
        } else {
          resolve(event.data.result);
        }
      };
      
      window.addEventListener('message', listener);
      
      // Post the message to the content script
      window.postMessage({
        type: 'THUNDER_REQUEST',
        id,
        method: args.method,
        params: args.params
      }, '*');
    });
  }
}

// Attach to window
(window as any).thunder = new ThunderProvider();
console.log('⚡ Thunder Provider injected into webpage');
