const rust = import('./pkg/crabmancake.js');
rust
    .then(m => {
        m.default();
        console.log("Hello from javascript");
    })
    .catch(console.error);

