import Block from "./Block";
import Color from "./Color";
import ProjectionMatrix from "./ProjectionMatrix";
import { Vertex, Vertex3D, drawTriangle } from "./shapes";

// represents a triangle slice of a block
interface Slice {
    pos: Vertex3D,  // position of topmost vertex
    id: number,  // which of the six slices of the hex projection this is; 0 is top right, then go clockwise
    parent: Block,
}

function drawSlice(s: Slice, projMatrix: ProjectionMatrix, offset: Vertex, context: CanvasRenderingContext2D) {
    let v3d: [Vertex3D, Vertex3D, Vertex3D];
    let color: Color;
    switch (s.id) {
        case 0:
            v3d = [
                s.pos,
                [s.pos[0] + 1, s.pos[1], s.pos[2]],
                [s.pos[0] + 1, s.pos[1] + 1, s.pos[2]],
            ];
            color = s.parent.color;
            break;
        case 1:
            v3d = [
                s.pos,
                [s.pos[0], s.pos[1], s.pos[2] - 1],
                [s.pos[0], s.pos[1] + 1, s.pos[2]],
            ];
            color = s.parent.color.scaled(0.8);
            break;
        case 2:
            v3d = [
                s.pos,
                [s.pos[0], s.pos[1] - 1, s.pos[2] - 1],
                [s.pos[0], s.pos[1], s.pos[2] - 1],
            ];
            color = s.parent.color.scaled(0.8);
            break;
        case 3:
            v3d = [
                s.pos,
                [s.pos[0], s.pos[1], s.pos[2] - 1],
                [s.pos[0] - 1, s.pos[1], s.pos[2] - 1],
            ];
            color = s.parent.color.scaled(0.9);
            break;
        case 4:
            v3d = [
                s.pos,
                [s.pos[0] + 1, s.pos[1], s.pos[2]],
                [s.pos[0], s.pos[1], s.pos[2] - 1],
            ];
            color = s.parent.color.scaled(0.9);
            break;
        case 5:
            v3d = [
                s.pos,
                [s.pos[0] + 1, s.pos[1] + 1, s.pos[2]],
                [s.pos[0], s.pos[1] + 1, s.pos[2]],
            ];
            color = s.parent.color;
            break;
        default:
            throw new Error("Invalid slice id");
    }
    const vertices = v3d.map((v) => {
        const v2d = [v[0] - v[2], v[1] - v[2]] as Vertex;
        const proj = projMatrix.proj(v2d);
        return [proj[0] + offset[0], proj[1] + offset[1]];
    }) as [Vertex, Vertex, Vertex];
    drawTriangle(
        context,
        {
            vertices,
            fill: color,
        }
    );
}

function posForId(id: number, pos: Vertex3D): Vertex3D {
    switch (id) {
        case 0:
            return pos;
        case 1:
            return [pos[0] + 1, pos[1], pos[2]];
        case 2:
            return [pos[0] + 1, pos[1] + 1, pos[2]];
        case 3:
            return [pos[0] + 1, pos[1] + 1, pos[2]];
        case 4:
            return [pos[0], pos[1] + 1, pos[2]];
        case 5:
            return pos;
        default:
            throw new Error("Invalid slice id");
    }
}

export default class Scene {
    private blocks: Block[] = [];
    constructor() { }

    addBlock(b: Block) {
        // // inserts block while preserving ordering
        // if (this.blocks.length === 0) {
        //     this.blocks = [b];
        //     return;
        // }
        // let left = 0;
        // let right = this.blocks.length - 1;
        // while (left <= right) {
        //     const mid = Math.floor((left + right) / 2);
        //     if (b.drawAfter(this.blocks[mid])) {
        //         left = mid + 1;
        //     } else {
        //         right = mid - 1;
        //     }
        // }
        // this.blocks = [...this.blocks.slice(0, left), b, ...this.blocks.slice(left)];
        this.blocks.push(b);
    }

    draw(projMatrix: ProjectionMatrix, offset: Vertex, context: CanvasRenderingContext2D) {
        const slices = new Map<string, Slice>();
        for (let b of this.blocks) {
            for (let id = 0; id < 6; id++) {
                const s: Slice = {
                    pos: posForId(id, b.origin),
                    id,
                    parent: b,
                };
                const key = `${s.pos}${s.id % 2}`;
                const current = slices.get(key);
                if (current === undefined || b.drawAfter(current.parent)) {
                    slices.set(key, s);
                }
            }
        }
        for (let [_, s] of slices) {
            drawSlice(s, projMatrix, offset, context);
        }
    }
}