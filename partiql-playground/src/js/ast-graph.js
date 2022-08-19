// This code is based on the following with customizations:
// - https://bl.ocks.org/d3noob/43a860bc0024792f8803bba8ca0d5ecd with some customizations
// - https://onestepcode.com/zoom-pan-effect-svg/

let treeData, i, duration, root, tree, diagonal, svg;
/**
 * Draws a Graph on SVG using the input json string.
 * @param jsonString The input json string.
 */
function drawGraph(jsonString) {
    if (jsonString.trim().length === 0) {
        return false;
    }
    treeData = getTree(jsonString);

    function zoom() {
        svg.attr("transform", "translate(" + d3.event.translate + ")scale(" + d3.event.scale + ")");
        console.log("translate: " + d3.event.translate + ", scale: " + d3.event.scale);
    }

    const margin = {top: 20, right: 120, bottom: 20, left: 120},
        width = 2000 - margin.right - margin.left,
        height = 700 - margin.top - margin.bottom;

    i = 0;
    duration = 750;

    tree = d3.layout.tree()
        .size([height, width]);

    diagonal = d3.svg.diagonal()
        .projection(function (d) {
            return [d.y, d.x];
        });

    root = treeData.children[0];
    root.x0 = height /2;
    root.y0 = 0;

    const nodes = tree.nodes(root).reverse();
    svg = d3.select('#graph').append('svg')
        .attr("viewBox", "0 0 " + (nodes.length * 50) + " " + (nodes.length * 20) )
        .attr("preserveAspectRatio", "xMidYMid meet")
        .attr("pointer-events", "all")
        .call(d3.behavior.zoom().on("zoom", function () {
            svg.attr("transform", "translate(" + d3.event.translate + ")" + " scale(" + d3.event.scale + ")")
        }))
        .append("g")
    update(root);
}

/**
 * A helper function for updating the d3 layout based on the input root node.
 * @param rootNode Graph's root node.
 */
function update(rootNode) {
    const nodes = tree.nodes(root).reverse(),
        links = tree.links(nodes);

    nodes.forEach(function (d) {
        d.y = d.depth * 180;
    });

    const node = svg.selectAll('g.node')
        .data(nodes, function (d) {
            return d.id || (d.id = ++i);
        });

    let nodeEnter = node.enter().append('g')
        .attr('class', 'node')
        .attr('transform', () => {
            return 'translate(' + rootNode.y0 + ',' + rootNode.x0 + ')';
        })
        .on('click', click);

    nodeEnter.append('circle')
        .attr('r', 1e-6)
        .style('fill', (d) => {
            return d._children ? 'lightsteelblue' : '#fff';
        });

    nodeEnter.append('text')
        .attr('x', (d) => {
            return d.children || d._children ? -13 : 13;
        })
        .attr('dy', '.35em')
        .attr('text-anchor', function (d) {
            return d.children || d._children ? 'end' : 'start';
        })
        .text(function (d) {
            return d.name;
        })
        .style('fill-opacity', 1e-6);

    const nodeUpdate = node.transition()
        .duration(duration)
        .attr('transform', (d) => {
            return 'translate(' + d.y + ',' + d.x + ')';
        });

    nodeUpdate.select('circle')
        .attr('r', 10)
        .style('fill', (d) => {
            return d._children ? 'lightsteelblue' : '#fff';
        });

    nodeUpdate.select('text')
        .style('fill-opacity', 1);

    const nodeExit = node.exit().transition()
        .duration(duration)
        .attr('transform', () => {
            return 'translate(' + rootNode.y + ',' + rootNode.x + ')';
        })
        .remove();

    nodeExit.select('circle')
        .attr('r', 1e-6);

    nodeExit.select('text')
        .style('fill-opacity', 1e-6);

    const link = svg.selectAll('path.link')
        .data(links, function (d) {
            return d.target.id;
        });

    link.enter().insert('path', 'g')
        .attr('class', 'link')
        .attr('d', () => {
            const o = {x: rootNode.x0, y: rootNode.y0};
            return diagonal({source: o, target: o});
        });

    link.transition()
        .duration(duration)
        .attr('d', diagonal);

    link.exit().transition()
        .duration(duration)
        .attr('d', () => {
            const o = {x: rootNode.x, y: rootNode.y};
            return diagonal({source: o, target: o});
        })
        .remove();

    nodes.forEach(function (d) {
        d.x0 = d.x;
        d.y0 = d.y;
    });
}

/**
 * Create a d3 tree based on the input json data.
 * @param jsonData
 * @returns A json object representing d3 tree.
 */
function getTree(jsonData) {
    let treeData = $.parseJSON(jsonData);
    const graph = {"name": "ast"};

    let children = buildTree(treeData);

    graph.children = children;
    treeData = $.parseJSON(JSON.stringify(graph));
    return treeData;
}

/**
 * Builds a d3 subtree based on the input jsonObject.
 * @param jsonData json Data to be using for building d3 Graph.
 * @returns An array representing the d3 Graph.
 */
function buildTree(jsonData) {
    let graph = []
    if (jsonData != null && typeof jsonData == 'object') {
        $.each(jsonData, function (k, v) {
            // Purge nodes that are null to reduce the graph size
            if (v != null) {
                let children = buildTree(v);
                graph.push({'name': k, 'children': children});
            }
        });
    } else {
        graph.push({'name': jsonData});
    }
    return graph;
}

function click(d) {
    if (d.children) {
        d._children = d.children;
        d.children = null;
    } else {
        d.children = d._children;
        d._children = null;
    }
    update(d);
}
