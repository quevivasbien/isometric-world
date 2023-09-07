import Block from "./Block";
import Color from "./Color";
import Matrix from "./Matrix";
import Scene from "./Scene";

function randn() {
    return Math.sqrt(-2 * Math.log(Math.random())) * Math.cos(2 * Math.PI * Math.random());
}

function linspace(start: number, end: number, length: number) {
    const arr = new Array(length).fill(start);
    const step = (end - start) / (length - 1);
    for (let i = 1; i < length; i++) {
        arr[i] += step * i;
    }
    return arr;
}

export function sigmoid(x: number) {
    return 1 / (1 + Math.exp(-x));
}

function dotgrad(
    x: number, y: number,
    xi: number, yi: number,
    gradsX: Matrix, gradsY: Matrix,
) {
    const dx = x - xi;
    const dy = y - yi;
    return dx * gradsX.get(yi, xi) + dy * gradsY.get(yi, xi);
}

function smoothstep(x: number) {
    return 6 * x ** 5 - 15 * x ** 4 + 10 * x ** 3;
}

function interpolate(x0: number, x1: number, w: number) {
    return x0 + smoothstep(w) * (x1 - x0);
}

function perlinAt(x: number, y: number, gradsX: Matrix, gradsY: Matrix) {
    const x0 = Math.floor(x);
    const x1 = x0 + 1;
    const y0 = Math.floor(y);
    const y1 = y0 + 1;
    
    const sx = x - x0;
    const sy = y - y0;

    const n0 = dotgrad(x, y, x0, y0, gradsX, gradsY);
    const n1 = dotgrad(x, y, x1, y0, gradsX, gradsY);
    const ix0 = interpolate(n0, n1, sx);

    const n2 = dotgrad(x, y, x0, y1, gradsX, gradsY);
    const n3 = dotgrad(x, y, x1, y1, gradsX, gradsY);
    const ix1 = interpolate(n2, n3, sx);

    return interpolate(ix0, ix1, sy);
}

function getGrads(height: number, width: number, gradInterval: number) {
    const gradWidth = Math.max(0, Math.ceil(width / gradInterval)) + 2;
    const gradHeight = Math.max(Math.ceil(height / gradInterval)) + 2;
    const gradX = new Array<number>(gradWidth * gradHeight);
    const gradY = new Array<number>(gradWidth * gradHeight);
    for (let i = 0; i < gradWidth * gradHeight; i++) {
        gradX[i] = randn();
        gradY[i] = randn();
    }
    return [new Matrix(gradX, gradHeight, gradWidth), new Matrix(gradY, gradHeight, gradWidth)];
}

export function perlin(height: number, width: number, gradInterval: number) {
    const [gradX, gradY] = getGrads(height, width, gradInterval);
    const ys = linspace(1 + Math.random(), gradX.rows - 1 - Math.random(), height);
    const xs = linspace(1 + Math.random(), gradX.cols - 1 - Math.random(), width);

    const data = new Array<number>(height * width);
    for (let i = 0; i < height; i++) {
        for (let j = 0; j < width; j++) {
            data[i * width + j] = perlinAt(xs[j], ys[i], gradX, gradY);
        }
    }
    return new Matrix(data, height, width);
}

export function perlinLayers(
    height: number, width: number,
    gradIntervals: number[], amplitudes?: number[]
) {
    if (amplitudes === undefined) {
        amplitudes = gradIntervals.map(() => 1);
    }
    else if (amplitudes.length !== gradIntervals.length) {
        throw new Error("Number of amplitudes must match number of grad intervals");
    }
    let outMatrix = new Matrix(new Array<number>(height * width).fill(0), height, width);
    for (let i = 0; i < gradIntervals.length; i++) {
        const perlinMatrix = perlin(height, width, gradIntervals[i]).map(x => x * (amplitudes as number[])[i]) as Matrix;
        outMatrix = outMatrix.mapElemwise((x, y) => x + y, perlinMatrix);
    }
    return outMatrix;
}

export function sceneFromHeightMap(heightMap: Matrix, minHeight: number) {
    heightMap = heightMap.map((x) => Math.floor(x));
    const scene = new Scene();
    for (let i = 0; i < heightMap.rows; i++) {
        for (let j = 0; j < heightMap.cols; j++) {
            const h = heightMap.get(i, j);
            for (let z = Math.floor(minHeight); z <= h; z++) {
                scene.addBlock(new Block([j, i, z], new Color(sigmoid(z)**0.5, 0.8 * (1 - sigmoid(z)), 0.4)));
            }
        }
    }
    return scene;
}
