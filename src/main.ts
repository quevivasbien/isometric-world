import './style.css';
import init, { render_test } from 'wasm';

const canvas = document.getElementById("canvas") as HTMLCanvasElement;
const context = canvas.getContext("2d") as CanvasRenderingContext2D;

canvas.width = window.innerWidth * 0.75;
canvas.height = canvas.width * 0.75;

function renderRandom() {
  const offsetX = Math.floor(canvas.width * 0.5);
  const offsetY = 50;
  const time0 = Date.now();
  const bytes = render_test(canvas.height, canvas.width, offsetX, offsetY, 150, 150, 12.);
  console.log(`Rendered in ${Date.now() - time0}ms`);
  const imageData = context.createImageData(canvas.width, canvas.height);
  imageData.data.set(bytes);
  context.putImageData(imageData, 0, 0);
}

function renderLoop() {
  renderRandom();
  setTimeout(renderLoop, 1000);
}

init().then(() => {
  renderLoop();
});
