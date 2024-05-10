import './style.css';

import { Attributes, Component, ComponentChild, ComponentChildren, Ref, render } from "preact";
import StreamScrollbox from "./stream_scrollbox";

import TextField from 'preact-material-components/TextField';
import 'preact-material-components/TextField/style.css'

const API = '127.0.0.1:7032/api/algorithms';


interface IAlgorithmState {
    num_nodes: number;
    edges: Edge[];
    graph_text: String;
}

interface IAlgorithmProps {}

class Edge {
    src: number;
    dest: number;
    distance: number;
    directed: boolean;

    constructor(src: number, dest: number, distance: number, directed: boolean) {
        this.src = src;
        this.dest = dest;
        this.distance = distance;
        this.directed = directed;
    }
}

enum EdgeChangeType {
    SRC = 0,
    DEST,
    DISTANCE,
    DIRECTED,
}

export class Algorithms extends Component<IAlgorithmProps, IAlgorithmState> {
    state: IAlgorithmState;
    textarea_graph: TextField;

    constructor() {
        super();
        this.state = {
            num_nodes: 0,
            edges: [],
            graph_text: ""
        };
    }

    increment_nodes = () => { 
        this.setState({
            ...this.state,
            num_nodes: this.state.num_nodes + 1,
        });
        console.log("Nodes (+): " + (this.state.num_nodes));
    }

    decrement_nodes = () => {
        if(this.state.num_nodes > 0) {
            this.setState({
                ...this.state,
                num_nodes: this.state.num_nodes - 1,
            });
            console.log("Nodes (-): " + (this.state.num_nodes));
        }
    }

    textarea_changed = (event: InputEvent) => {
        let textarea = event.target as HTMLTextAreaElement;
        let text = textarea.value;
        console.log("Textarea: " + text);
    }

    add_edge = () => {
        if(this.state.num_nodes > 0) {
            console.log("Adding edge #" + (this.state.edges.length + 1));
            // can only add edges if at least 1 node exists
            let new_edges = this.state.edges as Edge[];
            new_edges.push(new Edge(0, 0, 0, true));
            this.setState({
                ...this.state,
                edges: new_edges,
            });

            this.updateGraphText();
        } else {
            console.log("Cannot add edge: no nodes in graph!");
        }
    }

    changeEdge = (index: number, change_type: EdgeChangeType, event: Event) => {
        let select = event.target as HTMLSelectElement;
        let value = select.value;
        console.log("Edge #" + (index + 1) + ": change " + EdgeChangeType[change_type] + " to " + (change_type === EdgeChangeType.DIRECTED ? (event.target as HTMLInputElement).checked : value));
        let new_edges = this.state.edges as Edge[];

        switch(change_type) {
            case EdgeChangeType.SRC: {
                new_edges[index].src = parseInt(value);
                break;
            }
            case EdgeChangeType.DEST: {
                new_edges[index].dest = parseInt(value);
                break;
            }
            case EdgeChangeType.DISTANCE: {
                new_edges[index].distance = parseInt(value);
                break;
            }
            case EdgeChangeType.DIRECTED: {
                let checkbox = event.target as HTMLInputElement;
                new_edges[index].directed = checkbox.checked;
                break;
            }            
        }

        //console.log("New Edge attributes: " + JSON.stringify(new_edges[index]));

        this.setState({
            ...this.state,
            edges: new_edges,
        });

        this.updateGraphText();
    }

    updateGraphText() {
        console.log("Updating graph text... old text: " + this.state.graph_text);
        let new_graph_text = "";
        this.state.edges.forEach((edge, index) => {
            new_graph_text += "(" + edge.src + ", " + edge.dest + ", " + edge.distance + ")" + "\n";
        });
        this.setState({
            ...this.state,
            graph_text: new_graph_text,
        });
        if(this.textarea_graph != null) {
            this.textarea_graph.MDComponent.value = new_graph_text;
        }
    }

    render(): ComponentChild { 
        let node_component = this.createNodeComponent();
        let edge_component = this.createEdgeComponent(); 

        return (
            <div id="algorithms-outer">
                <h1>Algorithms</h1>
                <div id="algorithms-inner">
                    <div >
                        {node_component}
                        {edge_component}
                    </div>
                    <div id="algorithms-inner-spacer" />
                    <div id="algorithms-text-outer">
                        <TextField ref={textarea_graph => this.textarea_graph = textarea_graph} onInput={this.textarea_changed} textarea={true} height="500px" helperText="Edge-Definitions: (src, dst, dist)" helperTextPersistent></TextField>
                    </div>
                </div>
                {/*<StreamScrollbox socket_url={API}/>*/}
            </div>
        );
    }

    createNodeComponent() {
        return (
            <div id="algorithms-node-outer">
                <p class="inline" style="float: center; padding-left: 10px; padding-right: 10px;">
                    Nodes: {this.state.num_nodes}
                </p>
                <div id="algorithms-node-inner" class="inline">
                    <button id="algorithms-add-node" class="algorithms-button" onClick={this.increment_nodes}>+</button>
                    <button id="algorithms-node-remove" class="algorithms-button" onClick={this.decrement_nodes}>-</button>
                </div>
            </div>
        );
    }

    createEdgeComponent() {
        let node_options = [];
        for(let i = 0; i < this.state.num_nodes; i++) {
            node_options.push(<option value={i} style="height: 10px;">{i + 1}</option>)
        }

        return (
            <div id="algorithms-edge-outer">
                <button id="algorithms-edge-add" class="algorithms-button" onClick={this.add_edge}>Add Edge</button>
                { this.state.edges.length > 0 &&
                    <table id="algorithmns-edges-outer">
                        <tr>
                            <th>Edge</th>
                            <th>Source</th>
                            <th>Destination</th>
                            <th>Distance</th>
                            <th>Directed</th>
                        </tr>
                    { this.state.edges.map((edge, index) => { return (
                        <tr class="algorithms-edge">
                            <td>#{index + 1}:</td>
                            <td class="node-select">
                                <select  value={edge.src} onChange={(event) => { this.changeEdge(index, EdgeChangeType.SRC, event); }}>{node_options}</select>
                            </td>
                            <td class="node-select">
                                <select value={edge.dest} onChange={(event) => { this.changeEdge(index, EdgeChangeType.DEST, event); }}>{node_options}</select>
                            </td>
                            <td>
                                <input type="number" value={edge.distance} onChange={(event) => { this.changeEdge(index, EdgeChangeType.DISTANCE, event); }} style="width: 40px;" placeholder="Distance"></input>
                            </td>
                            <td>
                                <input type="checkbox" checked={edge.directed} onChange={(event) => { this.changeEdge(index, EdgeChangeType.DIRECTED, event); }} />
                            </td>
                        </tr>
                        ); })
                    }
                </table>
                }
            </div>
        );
    }
}

//const API = '127.0.0.1:7032/api/algorithms';
/*
    let test = await fetch('http://' + API, {
        method: "POST",
        headers: { 
            "Content-Type": "application/json",
        },
        body: JSON.stringify({ request_type: "list", content: {list_type: "graph"}})
    });
*/

