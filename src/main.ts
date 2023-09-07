import ProjectionMatrix from './ProjectionMatrix';
import './style.css';
import { perlinLayers, sceneFromHeightMap } from './terrain';

const canvas = document.getElementById("canvas") as HTMLCanvasElement;
const context = canvas.getContext("2d") as CanvasRenderingContext2D;


canvas.width = window.innerWidth * 0.75;
canvas.height = canvas.width * 0.75;

const projMatrix = new ProjectionMatrix(Math.PI / 3, 16);

const heightMap = perlinLayers(40, 40, [20, 8], [9, 7]);
const scene = sceneFromHeightMap(heightMap, -8);

scene.draw(projMatrix, [Math.floor(canvas.width * 0.5), 1], context);
