export default class Color {
    // represent a color with r, g, b values in range [0, 1)
    constructor(public r: number, public g: number, public b: number) { 
      
    }
    hex() {
      const r = Math.floor(this.r * 255).toString(16).padStart(2, "0");
      const g = Math.floor(this.g * 255).toString(16).padStart(2, "0");
      const b = Math.floor(this.b * 255).toString(16).padStart(2, "0");
      return "#" + r + g + b;
    }
    scaled(c: number) {
      // doesn't actually scale physically accurately but it's good enough for now
      return new Color(
        Math.min(Math.max(0, this.r * c), 1),
        Math.min(Math.max(0, this.g * c), 1),
        Math.min(Math.max(0, this.b * c), 1),
      )
    }
  }
  