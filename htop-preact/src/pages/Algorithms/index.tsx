import { render } from "preact";
import { useEffect, useRef } from "preact/hooks";
import { signal } from "@preact/signals";
import './style.css';

const API = '127.0.0.1:7032/api/algorithms';
const signal_scrollbox = signal("");

export function Algorithms() {
    let ref_testtext = useRef(null);
    let ref_scrollbox = useRef(null);
    
    useEffect(() => {
        signal_scrollbox.value = "";

        setInterval(async () => {
            if (!(ref_testtext.current == null)) {
                let test = await fetch('http://' + API, {
                    method: "POST",
                    headers: { 
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({ request_type: "list", list_type: "graph"})
                });
                let response = await test.json();
                console.log("Response: " + response);
                render(<>{"Response: " + response}</>, ref_testtext.current);
            }
        }, 5000);

        console.log("[ALGS] Connecting WebSocket to " + API + "/ws");
        const socket = new WebSocket('ws://' + API + '/ws/console');
        socket.addEventListener('message', (event) => {
            if(!(ref_testtext.current == null)) {
                signal_scrollbox.value = signal_scrollbox.value + event.data;
           } else {
                console.log("[ALGS] Closing websocket...");
                socket.close();
            }
        });


        window.addEventListener("unload", function () {
        if(socket.readyState == WebSocket.OPEN) {
            console.log("Closing websocket from window unload event...");
            socket.close();
        }
        });
 
    }, []);

    return (
        <div id="algorithms-outer">
            <h1>Algorithms</h1>
            <p id="algorithms-testoutput" ref={ref_testtext} />
            <div class="scrollbox" ref={ref_scrollbox} style="">{signal_scrollbox.value}</div>
        </div>
    );
}