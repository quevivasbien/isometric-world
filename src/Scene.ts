import Block from "./Block";
import ProjectionMatrix from "./ProjectionMatrix";
import { Vertex } from "./shapes";

export default class Scene {
    private blocks: Block[] = [];
    constructor() { }

    addBlock(b: Block) {
        // inserts block while preserving ordering
        if (this.blocks.length === 0) {
            this.blocks = [b];
            return;
        }
        let left = 0;
        let right = this.blocks.length - 1;
        while (left <= right) {
            const mid = Math.floor((left + right) / 2);
            if (b.drawAfter(this.blocks[mid])) {
                left = mid + 1;
            } else {
                right = mid - 1;
            }
        }
        this.blocks = [...this.blocks.slice(0, left), b, ...this.blocks.slice(left)];
    }

    draw(projMatrix: ProjectionMatrix, offset: Vertex, context: CanvasRenderingContext2D) {
        for (let b of this.blocks) {
            b.draw(projMatrix, offset, context);
        }
    }
}