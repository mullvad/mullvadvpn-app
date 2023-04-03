export interface LandData {
  // A flat array with vertice float values for OpenGL to render. Each group of three floats is one
  // vertice. So `(positions[x], positions[x+1], positions[x+2])` is one vertice where `x` is a
  // multiple of 3.
  positions: Array<number>;

  // The indexes in `positions` of the vertices that makes up the triangles of the earth.
  //
  // # Example
  //
  // If `[indices[0], indices[1], indices[2]] == [7, 2, 0]`,
  // then the vertices for the first triangle to draw is:
  // `vertice0 = [positions[7*3], positions[7*3+1], positions[7*3+2]]
  // `vertice1 = [positions[2*3], positions[2*3+1], positions[2*3+2]]
  // `vertice2 = [positions[0*3], positions[0*3+1], positions[0*3+2]]
  //
  // The `*3` part comes from the fact that the `indices` says which vertice, to pull from
  // `positions`, and each vertice occupies three elements in `positions`.
  triangleIndices: Array<number>;

  // Same format as `triangle_indices` but denotes the indexes for drawing the contours between
  // geographical entities/countries.
  contourIndices: Array<number>;
}

// This is very similar to `landData`, but it denotes the vertexes and indices for the sphere that
// is drawn as the ocean.
export interface OceanData {
  positions: Array<number>;
  indices: Array<number>;
}
