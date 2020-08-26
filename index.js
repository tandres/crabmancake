const rust = import('./pkg/crabmancake.js');

async function crab() {
    let mod = await rust;
    let pre_init = await mod.default()
    mod.cmc_init();
    let cmc_client = new mod.CmcClient();
    console.log("Hello from javascript");
    cmc_client.say_hello();
}

crab().catch(console.error);
