import { h, render } from 'https://esm.sh/preact';
import htm from 'https://esm.sh/htm';
// Initialize htm with Preact

const API = '/api/cpus';

const html = htm.bind(h);
const cpudivwidth = 300;

function getCPUInfo(cpuinfo) {
    return html`${cpuinfo.map((cpu, i) => 
        html`<div class=cpu>
        <p>CPU${i.toString().padStart(2, ' ')}: ${cpu.toFixed(1).toString().padStart(6, ' ')}</p>
        <div class="cpu-border inner" style="width: ${cpudivwidth}px"></div>
        <div class="cpu-fullbar inner" style="width: ${cpudivwidth}px"></div>
        <div class="cpu-percentage inner" style="width: ${cpu / 100 * cpudivwidth}px"></div>
        </div>`)
    }`;
}

document.addEventListener("DOMContentLoaded", () => {
    let i = 0;

    setInterval(async () => {
        let response = await fetch(API);
        if(response.status !== 200) {
            throw new Error('HTTP error! status: ${response.status}');
        }
        let json = await response.json();

        let heading = html`<h1>System name: ${json[0]}</h1>`;
        let ramusage = html`<p>RAM Usage: ${(json[2] / (1024**3)).toFixed(1)}GB (${Math.floor(100 * json[2]/json[1])}% of ${(json[1]/(1024**3)).toFixed()}GB)</p>`;

        let cpuinfo = getCPUInfo(json[4]);

        i += 1;
        render(html`<div>${heading}${ramusage}${cpuinfo}</div>`, document.body);
   }, 100);

});