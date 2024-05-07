import { render } from "preact";
import { useEffect, useRef } from "preact/hooks";
import './style.css';

const API = 'http://127.0.0.1:7032/api/cpus';

const cpudivwidth = 300;
const cpuspercolumn = 4;

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
                let heading = <h1 id="systemname">System name: {json['system_name']}</h1>;
                let hostname = <h2 id="hostname">Hostname: {json['host_name']}</h2>;
                let ramusage = <p id="ramusage">RAM Usage: {(json['used_memory'] / (1024**3)).toFixed(1)}GB ({Math.floor(100 * json['used_memory']/json['total_memory'])}% of {(json['total_memory']/(1024**3)).toFixed()}GB)</p>;
                let cpuinfo = getCPUInfo(json['cpu_usage']);
            
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
    let table_data = [];
    let divwidth = "width: " + cpudivwidth + "px";
    for (let row = 0; row < cpuspercolumn; row++) {
        let row_data = [];
        for (let col = 0; col < (cpuinfo.length / cpuspercolumn); col++) {
            let i = col * cpuspercolumn + row;
            let cpu = cpuinfo[i];
            row_data.push(
                <td class="cpu" id={"cpu-" + i} style={"width:"+cpudivwidth+"px"}>
                    <div class="cpu-border inner" style={divwidth}>
                        <p class="cpu-text inner">CPU{(i+1).toString().padStart(2, ' ')}: {cpu.toFixed(1).toString().padStart(6, ' ')}</p>
                    </div>
                    <div class="cpu-fullbar inner" style={divwidth}/>
                    <div class="cpu-percentage inner" style={"width: " + (cpu / 100 * cpudivwidth) + "px"} />
                </td>);
        }
        table_data.push(<tr id={"cpu-row-" + row}>{row_data}</tr>);
    }

    return(<table id="cpu-table">{table_data}</table>)
}
