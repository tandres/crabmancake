const rust = import('./pkg/crabmancake.js');
const canvas = document.getElementById('rustCanvas');
const gl = canvas.getContext("webgl", { antialias: true});

async function crab() {
    let mod = await rust;
    let pre_init = await mod.default()
    mod.cmc_init();

    const FPS_THROTTLE = 1000.0 / 30.0;
    const cmcClient = await new mod.CmcClient();
    const initialTime = Date.now();
    let lastDrawTime = -1;

    function render() {
        window.requestAnimationFrame(render);
        const currTime = Date.now();

        if (currTime >= lastDrawTime + FPS_THROTTLE) {
            lastDrawTime = currTime;
            let elapsedTime = currTime - initialTime;
            cmcClient.update(elapsedTime, canvas.height, canvas.width);
            cmcClient.render();
        }
    }
    render();
}

crab().catch(console.error);
