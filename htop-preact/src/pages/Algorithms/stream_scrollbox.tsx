import { Component } from "preact";
import './style.css';

interface IStreamScrollboxProps {
    socket_url: String;
}

interface IStreamScrollboxState {
    text: String;
}

export default class StreamScrollbox extends Component<IStreamScrollboxProps, IStreamScrollboxState>{
    API: String;
    ref_scrollbox: HTMLDivElement;
    socket: WebSocket;

    constructor(props: IStreamScrollboxProps) {
        super();
        this.API = props.socket_url;
        this.state = { text: "" };
    }
    
    componentDidMount(): void {
        this.setState({text: ""});

        console.log("[ALGS] Connecting WebSocket to " + this.API + "/ws");
        this.socket = new WebSocket('ws://' + this.API + '/ws/console');
        this.socket.addEventListener('message', (event) => {
            if(!(this.ref_scrollbox == null)) {
                this.setState({
                    text: this.state.text + event.data,
                });
           } else {
                console.log("[ALGS] Closing websocket...");
                this.socket.close();
            }
        });
    }

    componentWillUnmount(): void {
        if(this.socket.readyState == WebSocket.OPEN) {
            console.log("Closing websocket from window unload event...");
            this.socket.close();
        }
   }

    render() {
        return (
            <div class="scrollbox" ref={ref_scrollbox => this.ref_scrollbox = ref_scrollbox} style=""><p>{this.state.text}</p></div>
        );
    }
}
