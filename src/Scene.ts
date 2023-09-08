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
    if (s.id % 2 === 0) {
        v3d = [
            s.pos,
            [s.pos[0] + 1, s.pos[1], s.pos[2]],
            [s.pos[0] + 1, s.pos[1] + 1, s.pos[2]],
        ];
    } else {
        v3d = [
            s.pos,
            [s.pos[0] + 1, s.pos[1] + 1, s.pos[2]],
            [s.pos[0], s.pos[1] + 1, s.pos[2]],
        ];
    }
    let color: Color;
    if (s.id === 0 || s.id === 5) {
        color = s.parent.color;
    } else if (s.id === 1 || s.id === 2) {
        color = s.parent.color.scaled(0.8);
    } else {
        color = s.parent.color.scaled(0.9);
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

function getSliceInfo(b: Block, id: number): [string, Slice] {
    let pos = b.origin;
    switch (id) {
        case 0:
            break;
        case 1:
            pos = [pos[0], pos[1] - 1, pos[2] - 1];
            break;
        case 2:
            pos = [pos[0], pos[1], pos[2] - 1];
            break;
        case 3:
            pos = [pos[0], pos[1], pos[2] - 1];
            break;
        case 4:
            pos = [pos[0] - 1, pos[1], pos[2] - 1];
            break;  
        case 5:
            break;
        default:
            throw new Error("Invalid slice id");
    }
    pos = [0, pos[1] - pos[0], pos[2] - pos[0]];
    const key = `${pos[1]},${pos[2]},${id % 2}`;
    const slice: Slice = {
        pos,
        id,
        parent: b,
    };
    return [key, slice];
}

export default class Scene {
    private blocks: Block[] = [];
    constructor() { }

    addBlock(b: Block) {
        this.blocks.push(b);
    }

    draw(projMatrix: ProjectionMatrix, offset: Vertex, context: CanvasRenderingContext2D) {
        const slices = new Map<string, Slice>();
        for (let b of this.blocks) {
            for (let id = 0; id < 6; id++) {
                const [key, val] = getSliceInfo(b, id);
                const current = slices.get(key);
                if (current === undefined || b.drawAfter(current.parent)) {
                    slices.set(key, val);
                }
            }
        }
        for (let [_, s] of slices) {
            drawSlice(s, projMatrix, offset, context);
        }
    }
}