export default class Matrix {
    constructor(readonly data: number[], readonly rows: number, readonly cols: number) {
      if (data.length !== rows * cols) {
        throw new Error("rows time cols must equal length of data");
      }
    }
  
    static columnVec(v: number[]) {
      return new Matrix(v, v.length, 1);
    }
  
    get(i: number, j: number) {
      if (i < 0 || i >= this.rows) {
        throw new Error("row out of bounds");
      }
      if (j < 0 || j >= this.cols) {
        throw new Error("col out of bounds");
      }
      return this.data[i * this.cols + j];
    }
  
    mult(other: Matrix) {
      if (other.rows !== this.cols) {
        throw new Error("incompatible matrix dimensions");
      }
      const data = [];
      for (let i = 0; i < this.rows; i++) {
        for (let j = 0; j < other.cols; j++) {
          // compute entry for position i, j of output
          let v = 0;
          for (let k = 0; k < this.cols; k++) {
            v += this.get(i, k) * other.get(k, j);
          }
          data.push(v);
        }
      }
      return new Matrix(data, this.rows, other.cols);
    }
  }
  