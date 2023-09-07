import Color from './Color';
import ProjectionMatrix from './ProjectionMatrix';
import { Vertex, Vertex3D, drawTriangle } from './shapes';

export default class Block {
    constructor(public origin: Vertex3D, public color: Color) {}
  
    vertices() {
      return [
        [...this.origin],
        [this.origin[0] + 1, this.origin[1], this.origin[2]],  // bottom-right
        [this.origin[0], this.origin[1] + 1, this.origin[2]],  // bottom-left
        [this.origin[0] + 1, this.origin[1] + 1, this.origin[2]], // bottom
        [this.origin[0], this.origin[1], this.origin[2] + 1],  // top
        [this.origin[0] + 1, this.origin[1], this.origin[2] + 1],  // top-right
        [this.origin[0], this.origin[1] + 1, this.origin[2] + 1],  // top-left
        [this.origin[0] + 1, this.origin[1] + 1, this.origin[2] + 1],  // center
      ];
    }
  
    projection(projMatrix: ProjectionMatrix) {
      return this.vertices().map(([x, y, z]) => {
        const vertex2d: Vertex = [x - z, y - z];
        return projMatrix.proj(vertex2d);
      });
    }
  
    draw(projMatrix: ProjectionMatrix, offset: Vertex, context: CanvasRenderingContext2D) {
      const vertices2d = this.projection(projMatrix).map(([v0, v1]) => [v0 + offset[0], v1 + offset[1]] as Vertex);
      // draw triangles clockwise from top-right
      drawTriangle(
        context,
        {
          vertices: [vertices2d[4], vertices2d[5], vertices2d[7]],
          fill: this.color,
        }
      );
      drawTriangle(
        context,
        {
          vertices: [vertices2d[5], vertices2d[1], vertices2d[7]],
          fill: this.color.scaled(0.8),
        }
      );
      drawTriangle(
        context,
        {
          vertices: [vertices2d[1], vertices2d[3], vertices2d[7]],
          fill: this.color.scaled(0.8),
        }
      );
      drawTriangle(
        context,
        {
          vertices: [vertices2d[3], vertices2d[2], vertices2d[7]],
          fill: this.color.scaled(0.9),
        }
      );
      drawTriangle(
        context,
        {
          vertices: [vertices2d[2], vertices2d[6], vertices2d[7]],
          fill: this.color.scaled(0.9),
        }
      );
      drawTriangle(
        context,
        {
          vertices: [vertices2d[6], vertices2d[4], vertices2d[7]],
          fill: this.color,
        }
      );
    }
  
    drawAfter(other: Block) {
      // determine whether this block should be rendered after another block
      const [tx, ty, tz] = this.origin;
      const [ox, oy, oz] = other.origin;
      if (tz > oz) {
        return true;
      } else if (tz < oz) {
        return false;
      }
      if (ty > oy) {
        return true;
      } else if (ty < oy) {
        return false;
      }
      return tx > ox;
    }
  }
  