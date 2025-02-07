const os = require("os");
const path = require("path");
const { spawn, spawnSync } = require("child_process");
const { Builder, By, Capabilities } = require("selenium-webdriver");
const { expect } = import("chai");

let driver;

let tauriDriver;

const application = path.resolve(
    __dirname,
    '..',
    '..',
    '..',
    'target',
    'release',
    'the-dw-manager'
);

before(async function () {
    spawnSync('cargo', ['build', '--release'])
    tauriDriver = spawn(
        path.resolve(os.homedir(), '.cargo', 'bin', 'tauri-driver'),
        [],
        { stdio: [null, process.stdout, process.stderr] }
    )
    this.timeout(120000)
  
    const capabilities = new Capabilities()
    capabilities.set('tauri:options', { application })
    capabilities.setBrowserName('wry')
  
    // start the webdriver client
    driver = await new Builder()
      .withCapabilities(capabilities)
      .usingServer('http://127.0.0.1:4444/')
      .build()
});

after(async function () {
    await driver.quit()
    tauriDriver.kill()
});

describe("Downloads Manually", () => {
    it("Downloads a file", async () => {
        const btn = await driver.FindElement(By.id("download-from-url"));
        await btn.click();
        const urlField = await driver.FindElement(By.id("url-box"));
        await urlField.sendKeys("https://zoom.us/client/6.3.6.47101/zoomusInstallerFull.pkg?archType=arm64&amp_device_id=dd033b1e-515f-413a-be17-375b05f9b092");
        const downloadBtn = await driver.FindElement(By.id("download-btn"));
        await downloadBtn.click();
        const downloadsFolder = path.join(os.homedir(), 'Downloads');
        const downloadPath = path.join(downloadsFolder, 'zoomusInstallerFull.pkg');
        expect(await driver.wait(async () => {
            return require('fs').existsSync(downloadPath);
        }, 10000)).to.be.true;
    });
});