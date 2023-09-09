import ProjectionMatrix from './ProjectionMatrix';
import './style.css';
import { perlinLayers, sceneFromHeightMap } from './terrain';

const canvas = document.getElementById("canvas") as HTMLCanvasElement;
const context = canvas.getContext("2d") as CanvasRenderingContext2D;


canvas.width = window.innerWidth * 0.75;
canvas.height = canvas.width * 0.75;

// const projMatrix = new ProjectionMatrix(Math.PI / 3, 16);

// const heightMap = perlinLayers(40, 40, [20, 8], [9, 7]);
// const scene = sceneFromHeightMap(heightMap, -8);

// const time0 = Date.now();
// scene.draw(projMatrix, [Math.floor(canvas.width * 0.5), 20], context);
// console.log(`Rendered in ${Date.now() - time0}ms`);

import init, { render_test } from 'wasm';

init().then(() => {
  const bytes = render_test(canvas.height, canvas.width);
  const imageData = context.createImageData(canvas.width, canvas.height);
  imageData.data.set(bytes);
  context.putImageData(imageData, 0, 0);
});