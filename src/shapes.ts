import Color from "./Color";

export type Vertex = [number, number];
export type Vertex3D = [number, number, number];

export interface Triangle {
  vertices: [Vertex, Vertex, Vertex],
  fill?: Color,
  border?: Color,
}

export function drawTriangle(context: CanvasRenderingContext2D, tri: Triangle) {
  context.beginPath();
  const [v1, v2, v3] = tri.vertices.map(
    (vertex) => vertex.map((x) => Math.floor(x)) as Vertex
  );
  context.moveTo(...v1);
  context.lineTo(...v2);
  context.lineTo(...v3);
  context.closePath();
  if (tri.fill) {
    context.fillStyle = tri.fill.hex();
    context.fill();
  }
  if (tri.border) {
    context.strokeStyle = tri.border.hex();
    context.stroke();
  }
}
