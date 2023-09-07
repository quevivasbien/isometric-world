import Block from './Block';
import Color from './Color';
import ProjectionMatrix from './ProjectionMatrix';
import Scene from './Scene';
import './style.css';

const canvas = document.getElementById("canvas") as HTMLCanvasElement;
const context = canvas.getContext("2d") as CanvasRenderingContext2D;


canvas.width = window.innerWidth * 0.75;
canvas.height = canvas.width * 0.75;

const projMatrix = new ProjectionMatrix(Math.PI / 3, 16);
const scene = new Scene();

for (let i = -4; i < 4; i++) {
  for (let j = -4; j < 4; j++) {
    scene.addBlock(new Block([i, j, -1], new Color(0.2, 0.1, 0.2)));
  }
}
scene.addBlock(new Block([0, 0, 0], new Color(1, 0, 0)));
scene.addBlock(new Block([0, 1, 0], new Color(0.8, 0.1, 0)));
scene.addBlock(new Block([0, 0, 1], new Color(1, 0.4, 0)));

scene.draw(projMatrix, [Math.floor(canvas.width * 0.5), Math.floor(canvas.height * 0.5)], context);
