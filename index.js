const rust = import('./pkg/crabmancake.js');
const canvas = document.getElementById('rustCanvas');
const gl = canvas.getContext("webgl", { antialias: true});

async function crab() {
    let mod = await rust;
    let pre_init = await mod.default()
    mod.cmc_init();

    const FPS_THROTTLE = 1000.0 / 30.0;
    const cmcClient = new mod.CmcClient();
    const initialTime = Date.now();
    let lastDrawTime = -1;

    function render() {
        window.requestAnimationFrame(render);
        const currTime = Date.now();

        if (currTime >= lastDrawTime + FPS_THROTTLE) {
            lastDrawTime = currTime;
            if (window.innerHeight !== canvas.height || window.innerWidth !== canvas.width) {
                canvas.height = window.innerHeight;
                canvas.clientHeight = window.innerHeight;
                canvas.style.height = window.innerHeight;

                canvas.width = window.innerWidth;
                canvas.clientWidth = window.innerWidth;
                canvas.style.width = window.innerWidth;

                gl.viewport(0, 0, window.innerWidth, window.innerHeight);
            }
            let elapsedTime = currTime - initialTime;
            cmcClient.update(elapsedTime, window.innerHeight, window.innerWidth);
            cmcClient.render();
        }
    }
    render();
}

crab().catch(console.error);
