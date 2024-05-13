import './style.css';

import { Attributes, Component, ComponentChild, ComponentChildren, Ref, render } from "preact";
import StreamScrollbox from "./stream_scrollbox";
import { instance, Graph } from "@viz-js/viz";

import TextField from 'preact-material-components/TextField';
import 'preact-material-components/TextField/style.css'

const API = '127.0.0.1:7032/api/algorithms';


interface IAlgorithmState {
    num_nodes: number;
    edges: Edge[];
    graph_text: string;
}

interface IAlgorithmProps {}

class Edge {
    src: number;
    dest: number;
    distance: number;
    directed: boolean;
    valid: boolean;

    constructor(src: number, dest: number, distance: number, directed: boolean, valid: boolean) {
        this.src = src;
        this.dest = dest;
        this.distance = distance;
        this.directed = directed;
        this.valid = valid;
    }
}

enum EdgeChangeType {
    SRC = 0,
    DEST,
    DISTANCE,
    DIRECTED,
    DELETE,
}

class Algorithms extends Component<IAlgorithmProps, IAlgorithmState> {
    state: IAlgorithmState;
    //textarea_graph: TextField;
    textarea_graph: HTMLTextAreaElement;

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
        console.log("Textarea change event: processing text...");


    }

    add_edge = () => {
        if(this.state.num_nodes > 0) {
            console.log("Adding edge #" + (this.state.edges.length + 1));
            // can only add edges if at least 1 node exists
            let new_edges = this.state.edges as Edge[];
            new_edges.push(new Edge(-1, -1, 0, true, false));
            this.setState({
                ...this.state,
                edges: new_edges,
            });

            this.updateGraphText();
        } else {
            console.log("Cannot add edge: no nodes in graph!");
        }
    }

    resetEdges = () => {
        console.log("Resetting edges...");
        this.setState({
            ...this.state,
            edges: [],
        });
        this.textarea_graph.value = "";
        console.log("Edges reset! New state: " + JSON.stringify(this.state));
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
            case EdgeChangeType.DELETE: {
                console.log("Deleting edge #" + (index + 1));
                new_edges.splice(index, 1);
                break;
            } 
        }

        if(change_type != EdgeChangeType.DELETE) {
            let src = new_edges[index].src;
            let dest = new_edges[index].dest;
            new_edges[index].valid = src >= 0 && dest >= 0 && src < this.state.num_nodes && dest < this.state.num_nodes;
            console.log("Validity calculation for edge #" + (index + 1) + ": " + new_edges[index].valid + " (src: " + src + ", dest: " + dest + ") " + new_edges.length);
        }

        //console.log("New Edge attributes: " + JSON.stringify(new_edges[index]));

        this.setState({
            ...this.state,
            edges: new_edges,
        });

        this.updateGraphText();
    }

    updateGraphText() {
        console.log("Updating graph text...");
        let new_graph_text = "";
        let valid_edges = 0;
        this.state.edges.forEach((edge, index) => {
            if(!edge.valid) {
                return;
            }
            valid_edges++;
            let src = edge.src + 1;
            let dest = edge.dest + 1;
            new_graph_text += '(' + src + ', ' + dest + ', ' + edge.distance + ')' + (index == this.state.edges.length - 1 ? '' : '\n');
        });
        this.setState({
            ...this.state,
            graph_text: new_graph_text,
        });
        if(this.textarea_graph != null) {
            this.textarea_graph.value = new_graph_text;
            this.textarea_graph.style.maxHeight = (Math.max(50, (valid_edges + 1) * 20)).toString() + "px";
            this.textarea_graph.style.minHeight = (Math.max(valid_edges * 20, 50)).toString() + "px";
            this.textarea_graph.style.height = (valid_edges * 20).toString() + "px";
        }
    }

    createVizGraph() {
        let graph: Graph = {
            name: "Some Graph",
            strict: true,
            directed: true,
            nodes: [],
            edges: [],
        };

        for(let i = 0; i < this.state.num_nodes; i++) {
            graph.nodes.push({ name: (i + 1).toString() });
        }

        this.state.edges.forEach((edge) => {
            if(edge.valid) {
                let src = (edge.src + 1);
                let dest = (edge.dest + 1);
                let distance = edge.distance.toString();
                
                graph.edges.push({
                    tail: src.toString(),
                    head: dest.toString(),
                    attributes: {
                        label: distance,
                    }
                });
            }
        });
        return graph;
    }

    renderGraph = () => {
        console.log("Updating graph...");

        instance().then(viz => {
            const graph: Graph = this.createVizGraph();
            //const svg = viz.renderSVGElement("digraph { a -> b }");
            const svg = viz.renderSVGElement(graph);

            document.getElementById('graph').innerHTML = "";
            document.getElementById('graph').appendChild(svg);
            //render(svg, document.getElementById("graph"));
        });
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

    tryParseGraphtext(_event: Event) {
        console.log("Attempting to parse graph text...");
        let graph_text = this.textarea_graph.value;
        let edges = graph_text.split('\n');
        let new_edges = [];
        edges.forEach((edge) => {
            console.log("Trying to parse edge: " + edge);
            let edge_components = edge.split(',');
            console.log("Edge components: " + JSON.stringify(edge_components));
            if(edge_components.length == 3) {
                let src = parseInt(edge_components[0].substring(1));
                let dest = parseInt(edge_components[1]);
                let distance = parseInt(edge_components[2].substring(0, edge_components[2].length - 1));
                let is_valid = src > 0 && dest > 0 && src <= this.state.num_nodes && dest <= this.state.num_nodes;
                new_edges.push(new Edge(src, dest, distance, true, is_valid));
            }
        });

        console.log("Parsed edges: " + JSON.stringify(new_edges));

        /*this.setState({
            ...this.state,
            edges: new_edges,
        });*/
    }

    createEdgeComponent() {
        let textarea_component = this.createTextareaComponent();


        let node_options = [];
        node_options.push(<option value={-1} style="height: 10px;">-</option>)
        for(let i = 0; i < this.state.num_nodes; i++) {
            node_options.push(<option value={i} style="height: 10px;">{i + 1}</option>)
        }

        let node_interface = 
            <div id="algorithms-edge-outer">
                <table id="algorithms-edges-outer">
                    <tr>
                        <th class="algorithms-edge-numcol">Edge</th>
                        <th>Source</th>
                        <th>Destination</th>
                        <th>Distance</th>
                        <th>Directed</th>
                        <th></th>
                        <th></th>
                    </tr>
                { this.state.edges.map((edge, index) => { return (
                    <tr class="algorithms-edge">
                        <td class="algorithms-edge-numcol">#{index + 1}:</td>
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
                        <td class="algorithms-edge-validitycol">
                            <i class={ edge.valid ? "fa-solid fa-check" : "fa-solid fa-xmark"} />
                        </td>
                        <td><button class="btn-icon" onClick={(event) => { this.changeEdge(index, EdgeChangeType.DELETE, event)}}><i class="fa fa-trash"></i></button></td>
                    </tr>
                    ); })
                }
                </table>
            </div>


        return (<>
            <div id="algorithms-edgecontrol-outer">
                <button id="algorithms-edge-add" class="algorithms-button" onClick={this.add_edge}>Add Edge</button>
                <button id="algorithms-edge-reset" class="algorithms-button" onClick={this.resetEdges}>Reset Edges</button>
                <button id="algorithms-edge-parsegraph" class="algorithms-button" onClick={(event) => this.tryParseGraphtext(event)}>Parse Text</button>
                <button id="algorithms-edge-rendergraph" class="algorithms-button" onClick={(_) => this.renderGraph()}>Render</button>
            </div>
            <div id="algorithms-inner">
                { this.state.edges.length > 0 &&
                <>
                    <div>
                        {node_interface}
                    </div>
                    <div id="algorithms-inner-spacer" />
                    <div id="algorithms-text-outer">
                        {textarea_component}
                    </div>
                </>
                }
            </div>
        </>);
    }

    createTextareaComponent() {
        return (<>
            <div id="algorithms-textarea-outer">
                <textarea   ref={textarea_graph => this.textarea_graph = textarea_graph} 
                            class="algorithms-graph-textinput"
                            onInput={this.textarea_changed}
                            value={this.state.graph_text}
                            placeholder="Edge-Definitions: (src, dst, dist)"
                />
            </div>
        </>);
    }

    render(): ComponentChild { 
        let node_component = this.createNodeComponent();
        let edge_component = this.createEdgeComponent(); 

        return (
            <div id="algorithms-outer">
                <h1>Algorithms</h1>
                {node_component}
                {edge_component}
                <div id="graph"></div>
                {/*<StreamScrollbox socket_url={API}/>*/}
            </div>
        );
    }
}

export { Algorithms };

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