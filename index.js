const rust = import('./pkg/crabmancake.js');
const canvas = document.getElementById('rustCanvas');
const gl = canvas.getContext("webgl", { antialias: true });

rust.then(m => {
    m.default();
});

