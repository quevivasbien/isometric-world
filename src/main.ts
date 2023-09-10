import './style.css';
import init, { StateManager } from 'wasm';

const canvas = document.getElementById("canvas") as HTMLCanvasElement;
const context = canvas.getContext("2d") as CanvasRenderingContext2D;

canvas.width = window.innerWidth * 0.75;
canvas.height = canvas.width * 0.75;

function render(state: StateManager) {
  const time0 = Date.now();
  state.draw();
  const bytes = state.get_canvas();
  console.log(`Rendered in ${Date.now() - time0}ms`);
  const imageData = context.createImageData(canvas.width, canvas.height);
  imageData.data.set(bytes);
  context.putImageData(imageData, 0, 0);
}

function randomState() {
  const offsetX = Math.floor(canvas.width * 0.5);
  const offsetY = 50;
  const state = StateManager.new(
    150, 150, 
    new Uint32Array([20, 8]), new Float32Array([9, 7]),
    canvas.height, canvas.width, 12.,
  );
  state.shift(-offsetX, -offsetY);
  return state;
}

const STEP_SIZE = 20;

init().then(() => {
  let state = randomState();
  render(state);

  const shiftView = (dx: number, dy: number) => {
    state.shift(dx, dy);
    state.draw();
    render(state);
  };

  document.addEventListener('keypress', (e) => {
    switch (e.key) {
      case "w":
        shiftView(0, STEP_SIZE);
        break;
      case "s":
        shiftView(0, -STEP_SIZE);
        break;
      case "a":
        shiftView(STEP_SIZE, 0);
        break;
      case "d":
        shiftView(-STEP_SIZE, 0);
        break;
      case " ":
        let state = randomState();
        render(state);
        break;
    }
  });
});
