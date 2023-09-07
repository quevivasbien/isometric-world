import Matrix from "./Matrix";
import { Vertex } from "./shapes";

export default class ProjectionMatrix {
  readonly m: Matrix;

  constructor(alpha: number, w: number) {
    const theta = Math.PI / 2 - alpha;
    const phi = 2 * alpha - Math.PI / 2;
    this.m = new Matrix(
      [
        w * Math.cos(theta), -w * Math.cos(phi), 
        w * Math.sin(theta), w * Math.sin(phi),
      ],
      2, 2,
    );
  }

  proj(v: Vertex): Vertex {
    return this.m.mult(Matrix.columnVec(v)).data as Vertex;
  }
}
