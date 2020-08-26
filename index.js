const rust = import('./pkg/crabmancake.js');

async function crab() {
    let m = await rust;
    let l = await m.default()
    let cmc_client = new m.CmcClient();
    console.log("Hello from javascript");
    cmc_client.say_hello();
}

crab().catch(console.error);
