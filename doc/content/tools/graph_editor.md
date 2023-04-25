+++
title = "Graph Editor"
description = "A tool for editing control flow graphs and generate petgraph code."
date = 2023-03-18T17:48:52.457Z
updated = 2023-03-18T17:48:52.457Z
draft = false
+++

<div id="container">
    <div id="graph">
        <svg id="svg-graph" onmousedown="mousedown(event)" onmouseup="addNode(event)" oncontextmenu="rightclick()" contextmenu="rightclick()">
            <marker id="arrow" markerWidth="10" markerHeight="7"
    refX="0" refY="3.5" orient="auto">
                <polygon points="0 0, 10 3.5, 0 7" />
            </marker>
        </svg>
    </div>
    <div>
        <pre id="mermaid"></pre>
    </div>
    <div><pre id="rust"></pre></div>
    <div><pre id="come"></pre></div>
</div>

<script>
let nextNodeId = 0;
let draggingNode = null;
let draggingCircleId = null;
let dragStartX = null;
let dragStartY = null;
let firstNode = null;
let edges = [];
let lastMouses = null;
const svgNS = "http://www.w3.org/2000/svg";

document.getElementById("svg-graph").addEventListener("contextmenu", function(event) {
    rightclick();
    event.preventDefault();
    event.stopPropagation();
});

function mousePosition(event) {
    const svg = document.getElementById("svg-graph");
    const point = new DOMPoint(parseFloat(event.clientX), parseFloat(event.clientY));
    let transformedPoint = point.matrixTransform(svg.getScreenCTM().inverse());
    return [transformedPoint.x, transformedPoint.y];
}

function createText(text) {
    const textElement = document.createElementNS(svgNS, "text");
    textElement.setAttribute("text-anchor", "middle");
    textElement.setAttribute("dominant-baseline", "central");
    textElement.setAttribute("fill", "white");
    const textNode = document.createTextNode(text);
    textElement.appendChild(textNode);
    textElement.setAttribute("pointer-events", "none");
    return textElement;
}

function mousedown(event) {
    lastMouses = event.buttons;
}

function addNode(event) {
    if (draggingNode) {
        lastMouses = null;
        return;
    }
    if (lastMouses & 2) {
        lastMouses = null;
        return;
    }
    lastMouses = null;
    const svg = document.getElementById("svg-graph");
    const group = document.createElementNS(svgNS, "g");
    let [x, y] = mousePosition(event);
    group.setAttribute("transform", `translate(${x},${y})`);
    group.setAttribute("id", "node-" + nextNodeId.toString())
    const circle = document.createElementNS(svgNS, "circle");
    circle.setAttribute("r", 10);
    circle.setAttribute("fill", "red");
    group.appendChild(circle);
    group.appendChild(createText(nextNodeId));
    group.addEventListener("mousedown", (event) => {
        dragStart(event, group);
    });
    group.addEventListener("mousemove", (event) => {
        dragging(event, group);
    });
    group.addEventListener("mouseup", (event) => {
        dragEnd(event, group);
    });
    svg.appendChild(group);
    nextNodeId++;
    event.stopPropagation();
}

function dragStart(event, node) {
    const position = node.getAttribute("transform");
    const [x, y] = position.substring("translate(".length, position.length - 1).split(",").map(parseFloat);
    dragStartX = x;
    dragStartY = y;
    draggingNode = node;
    event.stopPropagation();
}

function dragging(event, node) {
    if (draggingNode === node) {
        let [x, y] = mousePosition(event);
        node.setAttribute("transform", `translate(${x},${y})`);
        edges.forEach(edge => {
            if (edge.first === node || edge.second === node) {
                const [sx, sy, ex, ey] = connectionLinePosition(edge.first, edge.second);
                edge.line.setAttribute('x1', sx);
                edge.line.setAttribute('y1', sy);
                edge.line.setAttribute('x2', ex);
                edge.line.setAttribute('y2', ey);
            }
        });
    }
    event.stopPropagation();
}

function dragEnd(event, node) {
    if (draggingNode === node) {
        const [x, y] = mousePosition(event);
        const dx = x - dragStartX;
        const dy = y - dragStartY;
        dragStartX = null;
        dragStartY = null;
        const considerAsClick = dx *dx + dy* dy < 50;
        if (considerAsClick) {
            if (firstNode === null) {
                firstNode = node;
                node.childNodes[0].setAttribute("fill", "green");
            } else if (firstNode !== null) {
                connect(firstNode, node);
                firstNode.childNodes[0].setAttribute("fill", "red");
                firstNode = null;
            }
        }
        draggingNode = null;
    }
    event.stopPropagation();
}

function rightclick() {
    if (firstNode !== null) {
        firstNode.childNodes[0].setAttribute("fill", "red");
        firstNode = null;
    }
    if (draggingNode !== null) {
        draggingNode = null;
    }
}

function connect(first, second) {
    const svg = document.getElementById("svg-graph");
    const line = document.createElementNS(svgNS, 'line');
    const [sx, sy, ex, ey] = connectionLinePosition(first, second);
    line.setAttribute('x1', sx);
    line.setAttribute('y1', sy);
    line.setAttribute('x2', ex);
    line.setAttribute('y2', ey);
    line.setAttribute('stroke', '#000');
    line.setAttribute("style", "stroke:#000; marker-end: url(#arrow);");
    svg.appendChild(line);
    edges.push({ first, second, line });
    renderMermaid();
    renderRust();
    renderComeIR();
}

function connectionLinePosition(firstNode, secondNode) {
    const firstPosition = firstNode.getAttribute("transform");
    const [x1, y1] = firstPosition.substring("translate(".length, firstPosition.length - 1).split(",").map(parseFloat);
    const secondePosition = secondNode.getAttribute("transform");
    const [x2, y2] = secondePosition.substring("translate(".length, secondePosition.length - 1).split(",").map(parseFloat);
    const r = 10;
    const dx = x2 - x1;
    const dy = y2 - y1;
    let angle = Math.atan(dy / dx);
    if (dx < 0) {
        angle += Math.PI;
    }
    const sx = x1 + r *Math.cos(angle);
    const sy = y1 + r* Math.sin(angle);
    const ex = x2 - (r + 10) *Math.cos(angle);
    const ey = y2 - (r + 10)* Math.sin(angle);
    return [sx, sy, ex, ey];
}

function renderMermaid() {
    const mermaid = document.getElementById("mermaid");
    let content = "graph TD \n";
    for ({ first, second, line } of edges) {
        const firstId = first.getAttribute("id").substring("node-".length);
        const secondId = second.getAttribute("id").substring("node-".length);
        content += `  ${firstId} --> ${secondId}\n`;
    }
    mermaid.innerHTML = content;
}

function renderRust() {
    const rust = document.getElementById("rust");
    let content = "let mut graph: DiGraph<_, _, usize> = DiGraph::default();\n";
    let edge_content = "";
    let nodes = new Set();
    for ({ first, second, line } of edges) {
        const firstId = first.getAttribute("id").substring("node-".length);
        const secondId = second.getAttribute("id").substring("node-".length);
        edge_content += `graph.add_edge(node_${firstId}, node_${secondId}, ());\n`;
        nodes.add(firstId);
        nodes.add(secondId);
    }
    for (let node of nodes) {
        content += `let node_${node} = graph.add_node(${node});\n`;
    }
    content += edge_content;
    rust.innerHTML = content;
}

function renderComeIR() {
    const come = document.getElementById("come");
    let content = `let function_definition = FunctionDefinition {
    header: ir::FunctionHeader {
        name: "f".to_string(),
        parameters: Vec::new(),
        return_type: data_type::Type::None,
    },
    content: vec![`;
    let nodes = new Set();
    for ({ first, second, line } of edges) {
        const firstId = first.getAttribute("id").substring("node-".length);
        const secondId = second.getAttribute("id").substring("node-".length);
        nodes.add(firstId);
        nodes.add(secondId);
    }
    for (let node of nodes) {
        content += `
        BasicBlock {
            name: Some("bb${node}".to_string()),
            content: vec![`;
        let to = edges.filter(({ first }) => {
            const firstId = first.getAttribute("id").substring("node-".length);
            return firstId === node;
        }).map(({ second }) => {
            const secondId = second.getAttribute("id").substring("node-".length);
            return secondId;
        });
        if (to.length === 0) {
            content += `Ret { value: None }.into()`;
        } else if (to.length === 1) {
            let target = to[0];
            content += `jump("bb${target}")`;
        } else if (to.length === 2) {
            let first = to[0];
            let second = to[1];
            content += `branch("bb${first}", "bb${second}")`;
        }
        content += `],
        },`
    }
    content += `
    ],
};`;
    come.innerHTML = content;
}
</script>

<style>
    #container {
        margin-left: -24px;
        margin-right: -24px;
        min-width: calc(100% + 48px);
        max-width: 100vw;
        display: flex;
        flex-wrap: wrap;
        overflow: scroll;
        justify-content: center;
        align-items: center;
    }
    #container>div {
        flex-grow: 0;
        flex-shrink: 0;
        padding: 4px;
    }
    #container>div>pre {
        padding: 0;
        margin: 0;
        overflow: scroll;
        max-width: 100vw;
    }
    #graph {
        width: fit-content;
    }
    #svg-graph {
        background: white;
        min-height: 300px;
    }
    @media (min-width: 576px) {
        #graph, #mermaid, #rust, #come {
            flex-grow: 1;
        }
    }
</style>
