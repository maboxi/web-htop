import { render } from "preact";
import { useEffect, useRef } from "preact/hooks";
import './style.css';

const API = 'http://127.0.0.1:7032/api/cpus';

const cpudivwidth = 300;

export function HTOP() {
    let ref_cpu = useRef(null);
    useEffect(() => {
        setInterval(async () => {
            if(!(ref_cpu.current == null)) {
                let response = await fetch(API);
                if(response.status !== 200) {
                    throw new Error('HTTP error! status: ${response.status}');
                }
                let json = await response.json();
                let heading = <h1>System name: {json[0]}</h1>;
                let hostname = <h2>Hostname: {json[5]}</h2>;
                let ramusage = <p>RAM Usage: {(json[2] / (1024**3)).toFixed(1)}GB ({Math.floor(100 * json[2]/json[1])}% of {(json[1]/(1024**3)).toFixed()}GB)</p>;
                let cpuinfo = getCPUInfo(json[4]);
            
                render(
                    <>
                        {heading}
                        {hostname}
                        {ramusage}
                        {cpuinfo}
                    </>,
                    ref_cpu.current);
            }
        }, 100);
    });
    return (<div class="htop" ref={ref_cpu}></div>);
}

function getCPUInfo(cpuinfo: number[]) {
    return (<div>{cpuinfo.map((cpu, i) => 
        <div class="cpu">
        <div class="cpu-border inner" style={"width: " + cpudivwidth + "px"}>
            <p class="cpu-text inner">CPU{(i+1).toString().padStart(2, ' ')}: {cpu.toFixed(1).toString().padStart(6, ' ')}</p>
        </div>
        <div class="cpu-fullbar inner" style={"width: " + cpudivwidth + "px"}></div>
        <div class="cpu-percentage inner" style={"width: " + (cpu / 100 * cpudivwidth) + "px"}></div>
        </div>
    )}
    </div>);
}
