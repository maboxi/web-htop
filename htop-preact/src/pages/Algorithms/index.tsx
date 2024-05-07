import { render } from "preact";
import { useEffect, useRef } from "preact/hooks";

const API = 'http://127.0.0.1:7032/api/algorithms';

export function Algorithms() {
    let ref = useRef(null);

    useEffect(() => {
        setInterval(async () => {
            if (!(ref.current == null)) {
                let test = await fetch(API, {
                    method: "POST",
                    headers: { 
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({ request_type: "list", list_type: "graph"})
                });
                let response = await test.json();
                console.log("Response: " + response);
                render(<>{"Response: " + response}</>, ref.current);

            }
        }, 5000);
    });

    return (
        <p ref={ref}>Algorithms</p>
    );
}