import './style.css';
import init, { StateManager } from 'wasm';

const canvas = document.getElementById("canvas") as HTMLCanvasElement;
const context = canvas.getContext("2d") as CanvasRenderingContext2D;

canvas.width = window.innerWidth * 0.75;
canvas.height = canvas.width * 0.75;

function render(state: StateManager, draw = true) {
  if (draw) {
    const time0 = Date.now();
    state.draw();
    console.log(`Rendered in ${Date.now() - time0}ms`);
  }
  const bytes = state.get_canvas();
  const imageData = context.createImageData(canvas.width, canvas.height);
  imageData.data.set(bytes);
  context.putImageData(imageData, 0, 0);
}

function randomState() {
  const state = StateManager.new(
    new Uint32Array([32, 16, 8]), new Float32Array([9, 7, 4]),
    canvas.height, canvas.width, 12,
    Math.floor(Math.random() * 2048 - 1024),
  );
  return state;
}

const STEP_SIZE = 20;

init().then(() => {
  let state = randomState();
  render(state);

  let needsRefresh = false;
  function requestMove(dir: 'up' | 'down' | 'left' | 'right') {
    switch(dir) {
      case 'up':
        state.shift_y(-STEP_SIZE);
        break;
      case 'down':
        state.shift_y(STEP_SIZE);
        break;
      case 'left':
        state.shift_x(-STEP_SIZE);
        break;
      case 'right':
        state.shift_x(STEP_SIZE);
        break;
    }
    render(state, false);
    needsRefresh = true;
  }

  document.addEventListener('keypress', (e) => {
    switch (e.key) {
      case "w":
        requestMove('up');
        break;
      case "s":
        requestMove('down');
        break;
      case "a":
        requestMove('left');
        break;
      case "d":
        requestMove('right');
        break;
      case " ":
        state = randomState();
        render(state);
        break;
    }
  });

  setInterval(() => {
    if (needsRefresh) {
      needsRefresh = false;
      render(state);
    }
  }, 1000);
});
