+++
title = "IR Breadboard"
description = "A tool for visualizing different information of given IR"
date = 2024-06-20T16:21:18.153Z
updated = 2024-06-20T16:21:18.153Z
draft = false
+++

<div id="container">
  <textarea id="code-input"></textarea>
  <div id="menu">
    <button id="dump-cfg">Dump CFG</button>
    <select name="optimize" id="optimize">
      <option value="MemoryToRegister">MemoryToRegister</option>
      <option value="RemoveLoadDirectlyAfterStore">RemoveLoadDirectlyAfterStore</option>
      <option value="RemoveOnlyOnceStore">RemoveOnlyOnceStore</option>
      <option value="RemoveUnusedRegister">RemoveUnusedRegister</option>
      <option value="TopologicalSort">TopologicalSort</option>
    </select>
    <button id="structural-analysis">Structural Analysis</button>
  </div>
  <div id="info">
    <pre class="hidden" id="output"></pre>
    <svg class="hidden" id="svg-graph">
    </svg>
  </div>
</div>

<style>
.hidden {
  display: none;
}
#container {
  display: flex;
  align-items: center;
  margin-left: -24px;
  margin-right: -24px;
}
#code-input {
  width: 40%;
  min-height: 100px;
  height: 800px;
  font-size: 12px;
  font-family: 'Zed Mono', 'Fira Code', 'Source Code Pro', 'Courier New', Courier, monospace;
}
#menu {
  width: 18%;
  min-height: 100px;
  font-size: 10pt;
}
#menu > * {
  width: 100%;
}
#info {
  width: 40%;
  height: 800px;
  overflow: scroll;
}
#svg-graph {
  background: white;
}
#svg-graph text {
  font-size: 10px;
  stroke: 0;
  font: 'Zed Mono', 'Fira Code', 'Source Code Pro', 'Courier New', Courier, monospace;
  fill: #000;
}
.clickable {
  cursor: pointer;
}
</style>

<script src="https://dagrejs.github.io/project/dagre/latest/dagre.min.js">
</script>
<script type="module">
import init, { parse, optimize, dump_control_flow_graph, structural_analysis } from "./ir_breadboard.js";

const svgNS = "http://www.w3.org/2000/svg";
const nodeRadius = 30;
let expanded = [[0]];

function fill_holes(graph) {
  let new_nodes = [];
  const length = graph.node_holes.length + graph.nodes.length;
  for (let i = 0; i < length; ++i) {
    if (graph.node_holes[0] === i) {
      graph.node_holes.shift();
      new_nodes.push(undefined);
    } else {
      new_nodes.push(graph.nodes.shift());
    }
  }
  graph.nodes = new_nodes;
}

function startsWith(arr, sequence) {
  if (arr.length < sequence.length) {
    return false;
  }
  for (let i = 0; i < sequence.length; i++) {
    if (arr[i] !== sequence[i]) {
      return false;
    }
  }
  return true;
}

function layoutAll(root_graph, path) {
  fill_holes(root_graph);
  let g = new dagre.graphlib.Graph();
  g.setGraph({marginx: nodeRadius, marginy: nodeRadius});
  g.setDefaultEdgeLabel(function() { return {}; });
  root_graph.nodes.forEach((node, index) => {
    path.push(index);

    if (node !== undefined
      && expanded.find(it => startsWith(it, path)) === undefined
      && node.Single === undefined) {
      // eg.
      // expanded: [[1, 0, 1, 0], [2, 2, 2]]
      // path: [1, 0, 1]: expanded, will not reach here
      // path: [2, 0]   : not expanded, reach heres
      g.setNode(index, {
        label: node,
        width: nodeRadius * 4,
        height: nodeRadius * 3
      });
    } else if (node !== undefined && node.Single !== undefined) {
      g.setNode(index, {
        label: node,
        width: nodeRadius * 2,
        height: nodeRadius * 2
      });
    } else if (node !== undefined && node.If !== undefined) {
      layoutAll(node.If.content.graph, path);
      g.setNode(index, {
        label: node,
        width: node.If.content.graph.layouted.graph().width,
        height: node.If.content.graph.layouted.graph().height
      });
    } else if (node !== undefined && node.Loop !== undefined) {
      layoutAll(node.Loop.graph, path);
      g.setNode(index, {
        label: node,
        width: node.Loop.graph.layouted.graph().width,
        height: node.Loop.graph.layouted.graph().height
      });
    } else if (node !== undefined && node.Block !== undefined) {
      layoutAll(node.Block.graph, path);
      g.setNode(index, {
        label: node,
        width: node.Block.graph.layouted.graph().width,
        height: node.Block.graph.layouted.graph().height
      });
    } else if (node !== undefined) {
      console.error("unreachable");
    }
    path.pop();
  });
  root_graph.edges.forEach(edge => {
    if (edge) {
      g.setEdge(edge[0], edge[1]);
    }
  });
  dagre.layout(g);
  root_graph.layouted = g;
  root_graph.path = path;
}

function renderExpandableGraph(element, subgraph, path) {
  subgraph.layouted.edges().forEach(function(e) {
    let edge = subgraph.layouted.edge(e);
    let points = edge.points;
    const path = document.createElementNS(svgNS, "path");
    path.setAttribute("d", `M ${points[0].x} ${points[0].y} Q${points[1].x} ${points[1].y} ${points[2].x} ${points[2].y}`);
    path.setAttribute("stroke", "black");
    path.setAttribute("fill", "transparent");
    path.setAttribute("style", "stroke:#000; marker-end: url(#arrow);");
    path.setAttribute("stroke-width", "1.5");
    let edge_info = subgraph.edges.find(it => it && it.from === parseInt(e.v) && it.to === parseInt(e.w));
    if (edge_info && edge_info.back) {
      path.setAttribute('stroke-dasharray', '1.5 1.5');
    }
    element.appendChild(path);
  });
  subgraph.layouted.nodes().forEach((node, index) => {
    path.push(parseInt(node));
    let node_object = subgraph.nodes[node];
    let node_layout = subgraph.layouted.node(node);
    let node_subgraph = null;
    if (node_object.If !== undefined) {
      node_subgraph = node_object.If.content.graph;
    } else if (node_object.Loop !== undefined) {
      node_subgraph = node_object.Loop.graph;
    } else if (node_object.Block !== undefined) {
      node_subgraph = node_object.Block.graph;
    }
    if (node_object.Single !== undefined) {
      const circle = document.createElementNS(svgNS, "circle");
      circle.setAttribute("r", nodeRadius);
      circle.setAttribute("fill", "red");
      circle.setAttribute("cx", node_layout.x);
      circle.setAttribute("cy", node_layout.y);
      element.appendChild(circle);
    } else if (node_subgraph !== null && node_subgraph.layouted === undefined) {
      const group = document.createElementNS(svgNS, "g");
      group.setAttribute("transform", `translate(${node_layout.x - nodeRadius * 2}, ${node_layout.y - nodeRadius * 1.5})`);

      const rect = document.createElementNS(svgNS, "rect");
      rect.setAttribute("width", nodeRadius * 4);
      rect.setAttribute("height", nodeRadius * 3);
      rect.setAttribute("fill", "red");
      group.appendChild(rect);

      const textElement = document.createElementNS(svgNS, "text");
      textElement.setAttribute("x", nodeRadius * 2);
      textElement.setAttribute("y", nodeRadius * 1.5);
      textElement.setAttribute("text-anchor", "middle");
      textElement.setAttribute("dominant-baseline", "central");
      textElement.setAttribute("pointer-events", "none");
      textElement.setAttribute("style", "font-size: 20px; fill: white;");
      const textNode = document.createTextNode("+");
      textElement.appendChild(textNode);
      group.appendChild(textElement);
      group.classList.add("clickable");

      element.appendChild(group);
      let pathCloned = [...path];
      element.addEventListener('click', (event) => {
        event.stopPropagation();
        let inserted = false;
        for (let current_expaned of expanded) {
          if (pathCloned !== undefined && startsWith(pathCloned, current_expaned)) {
            current_expaned.splice(0, current_expaned.length, ...pathCloned);
            inserted = true;
            break;
          }
        }
        if (!inserted) {
          expanded.push(path);
        }
        render_structural_analysis();
      });
    } else if (node_subgraph !== null) {
      const subElement = document.createElementNS(svgNS, "g");
      subElement.setAttribute("transform", `translate(${node_layout.x - node_subgraph.layouted.graph().width / 2}, ${node_layout.y - node_subgraph.layouted.graph().height / 2})`);

      const rect = document.createElementNS(svgNS, "rect");
      rect.setAttribute("x", 0);
      rect.setAttribute("y", 0);
      rect.setAttribute("width", node_subgraph.layouted.graph().width);
      rect.setAttribute("height", node_subgraph.layouted.graph().height);
      rect.setAttribute("fill", "transparent");
      rect.setAttribute("stroke", "black");
      rect.setAttribute("stroke-width", "1");
      subElement.appendChild(rect);

      const closeG = document.createElementNS(svgNS, "g");
      const closeRect = document.createElementNS(svgNS, "rect");
      closeRect.setAttribute("x", 0);
      closeRect.setAttribute("y", 0);
      closeRect.setAttribute("width", 32);
      closeRect.setAttribute("height", 24);
      closeRect.setAttribute("fill", "black");

      const textElement = document.createElementNS(svgNS, "text");
      textElement.setAttribute("x", 16);
      textElement.setAttribute("y", 10);
      textElement.setAttribute("text-anchor", "middle");
      textElement.setAttribute("dominant-baseline", "central");
      textElement.setAttribute("pointer-events", "none");
      textElement.setAttribute("style", "font-size: 20px; fill: white;");
      const textNode = document.createTextNode("-");
      textElement.appendChild(textNode);

      closeG.appendChild(closeRect);
      closeG.appendChild(textElement);
      closeG.classList.add("clickable");

      let pathCloned = [...path];
      closeG.addEventListener('click', (event) => {
        event.stopPropagation();
        for (let current_expaned of expanded) {
          if (startsWith(current_expaned, pathCloned)) {
            while (current_expaned.length !== pathCloned.length) {
              current_expaned.pop();
            }
            current_expaned.pop();
            break;
          }
        }
        render_structural_analysis();
      });

      subElement.appendChild(closeG);

      renderExpandableGraph(subElement, node_subgraph, path);
      element.appendChild(subElement);
    } else {
      console.error("unreachable");
    }
    path.pop();
  });
}

function render_structural_analysis() {
  const code = document.getElementById('code-input').value;
  let analysis_result = structural_analysis(code);
  fill_holes(analysis_result.graph);
  layoutAll(analysis_result.graph, [0]);
  const svg = document.getElementById("svg-graph");
  svg.setAttribute("width", analysis_result.graph.layouted.graph().width);
  svg.setAttribute("height", analysis_result.graph.layouted.graph().height);

  svg.innerHTML = `<marker id="arrow" markerWidth="5" markerHeight="3.5" refX="4" refY="1.75" orient="auto">
    <polygon points="0 0, 5 1.75, 0 3.5" />
  </marker>`;
  const group = document.createElementNS(svgNS, "g");
  group.setAttribute("transform", `translate(0, 0)`);
  renderExpandableGraph(group, analysis_result.graph, [0]);
  svg.appendChild(group);

  svg.classList.remove('hidden');
  const textOutput = document.getElementById("output");
  textOutput.classList.add('hidden');
}

init().then(() => {
  document.getElementById('dump-cfg').addEventListener('click', (event) => {
    event.stopPropagation();
    const code = document.getElementById('code-input').value;
    let cfg = dump_control_flow_graph(code);
    let g = new dagre.graphlib.Graph();
    g.setGraph({marginx: nodeRadius, marginy: nodeRadius});
    g.setDefaultEdgeLabel(function() { return {}; });
    cfg.nodes.forEach((node, index) => {
      g.setNode(
        index,
        {
          label: index,
          width: nodeRadius * 2,
          height: nodeRadius * 2
        }
      );
    });
    for (let edge of cfg.edges) {
      g.setEdge(edge.from, edge.to);
    }
    dagre.layout(g);
    const svg = document.getElementById("svg-graph");
    svg.setAttribute("width", g.graph().width);
    svg.setAttribute("height", g.graph().height);
    svg.innerHTML = `<marker id="arrow" markerWidth="5" markerHeight="3.5" refX="4" refY="1.75" orient="auto">
        <polygon points="0 0, 5 1.75, 0 3.5" />
    </marker>`;
    g.edges().forEach(function(e) {
      let edge = g.edge(e);
      let points = edge.points;
      const path = document.createElementNS(svgNS, "path");
      path.setAttribute("d", `M ${points[0].x} ${points[0].y} Q${points[1].x} ${points[1].y} ${points[2].x} ${points[2].y}`);
      path.setAttribute("stroke", "black");
      path.setAttribute("fill", "transparent");
      path.setAttribute("style", "stroke:#000; marker-end: url(#arrow);");
      path.setAttribute("stroke-width", "1.5");
      let edge_info = cfg.edges.find(it => it.from === parseInt(e.v) && it.to === parseInt(e.w));
      if (edge_info.back) {
        path.setAttribute('stroke-dasharray', '1.5 1.5');
      }
      svg.appendChild(path);
    });
    g.nodes().forEach(function(n) {
      const group = document.createElementNS(svgNS, "g");
      group.setAttribute("transform", `translate(${g.node(n).x}, ${g.node(n).y})`);
      const circle = document.createElementNS(svgNS, "circle");
      circle.setAttribute("r", nodeRadius);
      circle.setAttribute("fill", "red");
      group.appendChild(circle);

      const textElement = document.createElementNS(svgNS, "text");
      textElement.setAttribute("text-anchor", "middle");
      textElement.setAttribute("dominant-baseline", "central");
      textElement.setAttribute("fill", "white");
      const textNode = document.createTextNode(cfg.nodes[parseInt(n)]);
      textElement.appendChild(textNode);
      textElement.setAttribute("pointer-events", "none");
      group.appendChild(textElement);

      svg.appendChild(group);
    });
    svg.classList.remove('hidden');
    const textOutput = document.getElementById("output");
    textOutput.classList.add('hidden');
  });
  document.getElementById('optimize').addEventListener('change', () => {
    const pass = document.getElementById('optimize').value;
    const code = document.getElementById('code-input').value;
    const optimized = optimize(code, pass);
    const svg = document.getElementById("svg-graph");
    const textOutput = document.getElementById("output");
    textOutput.innerText = optimized;
    svg.classList.add('hidden');
    textOutput.classList.remove('hidden');
  });
  document.getElementById('structural-analysis').addEventListener('click', (event) => {
    event.stopPropagation();
    render_structural_analysis();
  });
});
</script>
