import { mat4 } from 'gl-matrix';

import { landData } from './land-data-50m';
import { oceanData } from './ocean-data';

type Color = Array<number>;

interface IndexBuffer {
  indexBuffer: WebGLBuffer;
  length: number;
}

interface ProgramInfo {
  program: WebGLProgram;
  attribLocations: {
    vertexPosition: GLint;
    vertexColor?: GLint;
  };
  uniformLocations: {
    color?: WebGLUniformLocation;
    projectionMatrix: WebGLUniformLocation;
    modelViewMatrix: WebGLUniformLocation;
  };
}

interface ZoomAnimation {
  endTime: number;
  compute(now: number): [number, number];
}

export enum ConnectionState {
  secure,
  unsecure,
}

// Color values for various components of the map.
const landColor: Color = [0.16, 0.302, 0.45, 1.0];
const oceanColor: Color = [0.098, 0.18, 0.271, 1.0];
// The color of borders between geographical entities
const contourColor: Color = oceanColor;

// The green color of the location marker when in the secured state
const locationMarkerSecureColor: Color = [0.267, 0.678, 0.302];
// The red color of the location marken when in the unsecured state
const locationMarkerUnsecureColor: Color = [0.89, 0.251, 0.224];

// Zoom is distance from earths center. 1.0 is at the surface.
// These constants define the zoom levels for the connected and disconnected states.
const disconnectedZoom = 1.45;
const connectedZoom = 1.35;

// Animations longer than this time will use the out-in zoom animation.
// Shorter animations will use the direct animation.
const zoomAnimationStyleTimeBreakpoint = 1.7;
// When animating with the out-in zoom animation, set the middle
// zoom point to this times the max start or end zoom levels.
const animationZoomoutFactor = 1.15;
// Never zoom out further than this.
const maxZoomout = Math.max(disconnectedZoom, connectedZoom) * animationZoomoutFactor;

// The min and max time an animation to a new location can take.
const animationMinTime = 1.3;
const animationMaxTime = 2.5;

// A geographical latitude, longitude coordinate in *degrees*.
// This class is also being abused as a 2D vector in some parts of the code.
export class Coordinate {
  public constructor(public lat: number, public long: number) {
    this.lat = lat;
    this.long = long;
  }

  // When treated as a 2D vector: Get the length of said vector.
  public length() {
    return Math.sqrt(this.lat * this.lat + this.long * this.long);
  }

  // When treated as a 2D vector: Scale that vector by `r`
  public scale(r: number) {
    return new Coordinate(this.lat * r, this.long * r);
  }

  // When treated as a 2D vector: Add two vectors together returning the sum
  public add(otherCoordinate: Coordinate) {
    return new Coordinate(this.lat + otherCoordinate.lat, this.long + otherCoordinate.long);
  }
}

// Class for drawing earth. Relies on a global constant `landData` being defined.
// It also relies on a global constant `oceanData` with the following format:
class Globe {
  private static vsSource = `
    attribute vec3 aVertexPosition;

    uniform vec4 uColor;
    uniform mat4 uModelViewMatrix;
    uniform mat4 uProjectionMatrix;

    varying lowp vec4 vColor;

    void main(void) {
      gl_Position = uProjectionMatrix * uModelViewMatrix * vec4(aVertexPosition, 1.0);
      vColor = uColor;
    }
  `;

  private static fsSource = `
    varying lowp vec4 vColor;

    void main(void) {
      gl_FragColor = vColor;
    }
  `;

  private landVertexBuffer: WebGLBuffer;
  private landContourIndexBuffer: IndexBuffer;
  private landTriangleIndexBuffer: IndexBuffer;
  private oceanVertexBuffer: WebGLBuffer;
  private oceanIndexBuffer: IndexBuffer;

  private programInfo: ProgramInfo;

  public constructor(private gl: WebGL2RenderingContext) {
    this.landVertexBuffer = initArrayBuffer(gl, landData.positions);
    this.oceanVertexBuffer = initArrayBuffer(gl, oceanData.positions);

    this.landContourIndexBuffer = initIndexBuffer(gl, landData.contourIndices);
    this.landTriangleIndexBuffer = initIndexBuffer(gl, landData.triangleIndices);
    this.oceanIndexBuffer = initIndexBuffer(gl, oceanData.indices);

    const shaderProgram = initShaderProgram(gl, Globe.vsSource, Globe.fsSource);
    this.programInfo = {
      program: shaderProgram,
      attribLocations: {
        vertexPosition: gl.getAttribLocation(shaderProgram, 'aVertexPosition'),
      },
      uniformLocations: {
        color: gl.getUniformLocation(shaderProgram, 'uColor')!,
        projectionMatrix: gl.getUniformLocation(shaderProgram, 'uProjectionMatrix')!,
        modelViewMatrix: gl.getUniformLocation(shaderProgram, 'uModelViewMatrix')!,
      },
    };
  }

  public draw(projectionMatrix: mat4, viewMatrix: mat4) {
    const globeViewMatrix = mat4.clone(viewMatrix);

    this.gl.useProgram(this.programInfo.program);

    // Draw country contour lines
    drawBufferElements(
      this.gl,
      this.programInfo,
      projectionMatrix,
      globeViewMatrix,
      this.landVertexBuffer,
      this.landContourIndexBuffer,
      contourColor,
      this.gl.LINES,
    );

    // We scale down to render the land triangles behind/under the country contour lines.
    mat4.scale(
      globeViewMatrix, // destination matrix
      globeViewMatrix, // matrix to scale
      [0.9999, 0.9999, 0.9999], // amount to scale
    );

    // Draw land triangles.
    drawBufferElements(
      this.gl,
      this.programInfo,
      projectionMatrix,
      globeViewMatrix,
      this.landVertexBuffer,
      this.landTriangleIndexBuffer,
      landColor,
      this.gl.TRIANGLES,
    );

    // Draw the ocean as a sphere just beneath the land.
    drawBufferElements(
      this.gl,
      this.programInfo,
      projectionMatrix,
      globeViewMatrix,
      this.oceanVertexBuffer,
      this.oceanIndexBuffer,
      oceanColor,
      this.gl.TRIANGLES,
    );
  }
}

// Class for rendering a location marker on a given coordinate on the globe.
class LocationMarker {
  private static vsSource = `
    attribute vec3 aVertexPosition;
    attribute vec4 aVertexColor;

    uniform mat4 uModelViewMatrix;
    uniform mat4 uProjectionMatrix;

    varying lowp vec4 vColor;

    void main(void) {
        gl_Position = uProjectionMatrix * uModelViewMatrix * vec4(aVertexPosition, 1.0);
        vColor = aVertexColor;
    }
    `;

  private static fsSource = `
    varying lowp vec4 vColor;

    void main(void) {
        gl_FragColor = vColor;
    }
  `;

  private programInfo: ProgramInfo;
  private ringPositionCount: Array<number>;
  private positionBuffer: WebGLBuffer;
  private colorBuffer: WebGLBuffer;

  public constructor(private gl: WebGL2RenderingContext, color: Color) {
    const white = [1.0, 1.0, 1.0];
    const black = [0.0, 0.0, 0.0];
    const rings = [
      circleFanVertices(32, 0.5, [0.0, 0.0, 0.0], [...color, 0.4], [...color, 0.4]), // Semi-transparent outer
      circleFanVertices(16, 0.28, [0.0, -0.05, 0.00001], [...black, 0.55], [...black, 0.0]), // shadow
      circleFanVertices(32, 0.185, [0.0, 0.0, 0.00002], [...white, 1.0], [...white, 1.0]), // white ring
      circleFanVertices(32, 0.15, [0.0, 0.0, 0.00003], [...color, 1.0], [...color, 1.0]), // Center colored circle
    ];
    this.ringPositionCount = rings.map((r) => r.positions.length);
    this.positionBuffer = initArrayBuffer(gl, rings.map((r) => r.positions).flat());
    this.colorBuffer = initArrayBuffer(gl, rings.map((r) => r.colors).flat());

    const shaderProgram = initShaderProgram(gl, LocationMarker.vsSource, LocationMarker.fsSource);
    this.programInfo = {
      program: shaderProgram,
      attribLocations: {
        vertexPosition: gl.getAttribLocation(shaderProgram, 'aVertexPosition'),
        vertexColor: gl.getAttribLocation(shaderProgram, 'aVertexColor'),
      },
      uniformLocations: {
        projectionMatrix: gl.getUniformLocation(shaderProgram, 'uProjectionMatrix')!,
        modelViewMatrix: gl.getUniformLocation(shaderProgram, 'uModelViewMatrix')!,
      },
    };
  }

  public draw(projectionMatrix: mat4, viewMatrix: mat4, coordinate: Coordinate, size: number) {
    const modelViewMatrix = mat4.clone(viewMatrix);

    this.gl.useProgram(this.programInfo.program);

    const [theta, phi] = coordinates2thetaphi(coordinate);
    mat4.rotateY(modelViewMatrix, modelViewMatrix, theta);
    mat4.rotateX(modelViewMatrix, modelViewMatrix, -phi);

    mat4.scale(modelViewMatrix, modelViewMatrix, [size, size, 1.0]);
    mat4.translate(modelViewMatrix, modelViewMatrix, [0.0, 0.0, 1.0001]);

    {
      this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.positionBuffer);
      this.gl.vertexAttribPointer(
        this.programInfo.attribLocations.vertexPosition,
        3, // num components
        this.gl.FLOAT, // type
        false, // normalize
        0, // stride
        0, // offset
      );
      this.gl.enableVertexAttribArray(this.programInfo.attribLocations.vertexPosition);
    }
    {
      this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.colorBuffer);
      this.gl.vertexAttribPointer(
        this.programInfo.attribLocations.vertexColor!,
        4, // num components
        this.gl.FLOAT, // type
        false, // normalize
        0, // stride
        0, // offset
      );
      this.gl.enableVertexAttribArray(this.programInfo.attribLocations.vertexColor!);
    }

    // Set the shader uniforms
    this.gl.uniformMatrix4fv(
      this.programInfo.uniformLocations.projectionMatrix,
      false,
      projectionMatrix,
    );
    this.gl.uniformMatrix4fv(
      this.programInfo.uniformLocations.modelViewMatrix,
      false,
      modelViewMatrix,
    );

    let offset = 0;
    for (let i = 0; i < this.ringPositionCount.length; i++) {
      const numVertices = this.ringPositionCount[i] / 3;
      this.gl.drawArrays(this.gl.TRIANGLE_FAN, offset, numVertices);
      offset += numVertices;
    }
  }
}

// Class for computing a smooth linear interpolation from
// `startCoordinate` (2D coordinate in degrees), along `path` (2D vector),
// starting at zoom level `startZoom` and ending in `endZoom`.
// starting at time `startTime` (usually now() at the time of creating an instance),
// and animating for `duration` seconds
class SmoothLerp {
  public constructor(
    private startCoordinate: Coordinate,
    private path: Coordinate,
    private startTime: number,
    private duration: number,
  ) {}

  // Computes and returns the coordinate as well as the zoom level and the smoothened transition
  // ratio of this lerp operation.
  public compute(now: number): [Coordinate, number] {
    const animationRatio = Math.min(Math.max((now - this.startTime) / this.duration, 0.0), 1.0);
    const smoothAnimationRatio = smoothTransition(animationRatio);
    const position = this.startCoordinate.add(this.path.scale(smoothAnimationRatio));
    return [position, smoothAnimationRatio];
  }
}

// Zooms from startZoom to endZoom via a midpoint that is `animationZoomoutFactor` times higer up
// than max(startZoom, endZoom).
class SmoothZoomOutIn implements ZoomAnimation {
  private middleZoom: number;

  public constructor(
    private startZoom: number,
    private endZoom: number,
    private startTime: number,
    private duration: number,
  ) {
    this.middleZoom = Math.min(Math.max(startZoom, endZoom) * animationZoomoutFactor, maxZoomout);
  }

  get endTime(): number {
    return this.startTime + this.duration;
  }

  public compute(now: number): [number, number] {
    const animationRatio = Math.min(Math.max((now - this.startTime) / this.duration, 0.0), 1.0);
    // Linear animation ratio 0-1. 0.0-0.5 means zooming out and 0.5-1.0 means zooming in
    if (animationRatio <= 0.5) {
      const smoothAnimationRatio = smoothTransition(animationRatio * 2);
      return [
        this.startZoom + smoothAnimationRatio * (this.middleZoom - this.startZoom),
        animationRatio,
      ];
    } else {
      const smoothAnimationRatio = smoothTransition((animationRatio - 0.5) * 2);
      return [
        this.middleZoom - smoothAnimationRatio * (this.middleZoom - this.endZoom),
        animationRatio,
      ];
    }
  }
}

// Zooms from startZoom to endZoom directly in a smooth manner.
class SmoothZoomDirect implements ZoomAnimation {
  public constructor(
    private startZoom: number,
    private endZoom: number,
    private startTime: number,
    private duration: number,
  ) {}

  get endTime(): number {
    return this.startTime + this.duration;
  }

  public compute(now: number): [number, number] {
    const animationRatio = Math.min(Math.max((now - this.startTime) / this.duration, 0.0), 1.0);
    const smoothAnimationRatio = smoothTransition(animationRatio);
    return [
      this.startZoom + smoothAnimationRatio * (this.endZoom - this.startZoom),
      animationRatio,
    ];
  }
}

export default class Map {
  private globe: Globe;
  private locationMarkerSecure: LocationMarker;
  private locationMarkerUnsecure: LocationMarker;

  private coordinate: Coordinate;
  private zoom: number;
  private connectionState: ConnectionState;
  private targetCoordinate: Coordinate;

  private animations: Array<SmoothLerp>;
  private zoomAnimations: Array<ZoomAnimation>;

  public constructor(
    gl: WebGL2RenderingContext,
    startCoordinate: Coordinate,
    connectionState: ConnectionState,
    private animationEndListener?: () => void,
  ) {
    this.globe = new Globe(gl);
    this.locationMarkerSecure = new LocationMarker(gl, locationMarkerSecureColor);
    this.locationMarkerUnsecure = new LocationMarker(gl, locationMarkerUnsecureColor);

    this.coordinate = startCoordinate;
    this.zoom = connectionState === ConnectionState.secure ? connectedZoom : disconnectedZoom;
    this.connectionState = connectionState;
    // `targetCoordinate` is the same as `coordinate` when no animation is in progress.
    // This is where the location marker is drawn.
    this.targetCoordinate = startCoordinate;
    // An array of smooth lerps between coordinates. Empty when no animation is in progress.
    this.animations = [];
    // An array of zoom animations. Empty when no animation is in progress.
    this.zoomAnimations = [];
  }

  // Move the location marker to `newCoordinate` (with state `connectionState`) and queue
  // animation to move to that coordinate.
  public setLocation(newCoordinate: Coordinate, connectionState: ConnectionState, now: number) {
    const endZoom = connectionState ? connectedZoom : disconnectedZoom;

    // Only perform a coordinate animation if the new coordinate is
    // different from the current position/latest ongoing animation.
    // If the new coordinate is the same as the current target, we just
    // queue a zoom animation.
    if (newCoordinate !== this.targetCoordinate) {
      const path = shortestPath(this.coordinate, newCoordinate);

      // Compute animation time as a function of movement distance. Clamp the
      // duration range between animationMinTime and animationMaxTime
      const duration = Math.min(Math.max(path.length() / 20, animationMinTime), animationMaxTime);

      this.animations.push(new SmoothLerp(this.coordinate, path, now, duration));
      if (duration > zoomAnimationStyleTimeBreakpoint) {
        this.zoomAnimations.push(new SmoothZoomOutIn(this.zoom, endZoom, now, duration));
      } else {
        this.zoomAnimations.push(new SmoothZoomDirect(this.zoom, endZoom, now, duration));
      }
    } else {
      let duration = animationMinTime;
      // If an animation is in progress, make sure our zoom animation ends at the same time.
      // Just makes a smooth transition from one zoom end state to the other.
      if (this.zoomAnimations.length > 0) {
        const lastZoomAnimation = this.zoomAnimations[this.zoomAnimations.length - 1];
        duration = Math.max(lastZoomAnimation.endTime - now, animationMinTime);
      }
      this.zoomAnimations.push(new SmoothZoomDirect(this.zoom, endZoom, now, duration));
    }

    this.connectionState = connectionState;
    this.targetCoordinate = newCoordinate;
  }

  // Render the map for the time `now`.
  public draw(projectionMatrix: mat4, now: number) {
    this.updatePosition(now);
    this.updateZoom(now);

    if (this.animations.length === 0 && this.zoomAnimations.length === 0) {
      this.animationEndListener?.();
    }

    const viewMatrix = mat4.create();

    // Move the camera back `this.zoom` away from the center of the globe.
    mat4.translate(
      viewMatrix, // destination matrix
      viewMatrix, // matrix to translate
      [0.0, 0.0, -this.zoom],
    );

    // Rotate the globe so the camera ends up looking down on `this.coordinate`.
    const [theta, phi] = coordinates2thetaphi(this.coordinate);
    mat4.rotateX(viewMatrix, viewMatrix, phi);
    mat4.rotateY(viewMatrix, viewMatrix, -theta);

    this.globe.draw(projectionMatrix, viewMatrix);

    // Draw the appropriate location marker depending on our connection state.
    const locationMarker =
      this.connectionState === ConnectionState.secure
        ? this.locationMarkerSecure
        : this.locationMarkerUnsecure;
    locationMarker.draw(projectionMatrix, viewMatrix, this.targetCoordinate, 0.03 * this.zoom);
  }

  // Private function that just updates internal animation state to match with time `now`.
  public updatePosition(now: number) {
    if (this.animations.length === 0) {
      return;
    }

    // Compute lerp position and ratio of the newest animation
    const lastAnimation = this.animations[this.animations.length - 1];
    let [coordinate, ratio] = lastAnimation.compute(now);
    if (ratio >= 1.0) {
      // Animation is done. We can empty the animations array
      this.animations = [];
    }

    // Loop through all previous animations (that are still in progress) backwards and
    // lerp between them to compute our actual location.
    for (let i = this.animations.length - 2; i >= 0; i--) {
      const [previousPoint, animationRatio] = this.animations[i].compute(now);
      coordinate = lerpCoordinate(previousPoint, coordinate, ratio);
      // If this animation is finished, none of the animations [0, i) will have any effect,
      // so they can be pruned
      if (animationRatio >= 1.0 && i > 0) {
        this.animations = this.animations.slice(i, this.animations.length);

        break;
      }
      ratio = animationRatio;
    }

    // Set our coordinate and zoom to the values interpolated from all ongoing animations.
    this.coordinate = coordinate;
  }

  // Private function that updates the current zoom level according to ongoing animations.
  updateZoom(now: number) {
    if (this.zoomAnimations.length === 0) {
      return;
    }

    const lastZoomAnimation = this.zoomAnimations[this.zoomAnimations.length - 1];
    let [zoom, ratio] = lastZoomAnimation.compute(now);

    if (ratio >= 1.0) {
      // Animation is done. We can empty the animations array
      this.zoomAnimations = [];
    }

    // Loop through all previous animations (that are still in progress) backwards and
    // lerp between them to compute our actual location.
    for (let i = this.zoomAnimations.length - 2; i >= 0; i--) {
      const [previousZoom, animationRatio] = this.zoomAnimations[i].compute(now);
      zoom = lerp(previousZoom, zoom, ratio);
      // If this animation is finished, none of the animations [0, i) will have any effect,
      // so they can be pruned
      if (animationRatio >= 1.0 && i > 0) {
        this.zoomAnimations = this.zoomAnimations.slice(i, this.zoomAnimations.length);
        break;
      }
      ratio = animationRatio;
    }

    // Set our coordinate and zoom to the values interpolated from all ongoing animations.
    this.zoom = zoom;
  }
}

// Draws primitives of type `mode` (TRIANGLES, LINES etc) using vertex positions from
// `positionBuffer` at indices in `indices` with the color `color` and using the shaders in
// `programInfo`.
function drawBufferElements(
  gl: WebGL2RenderingContext,
  programInfo: ProgramInfo,
  projectionMatrix: mat4,
  modelViewMatrix: mat4,
  positionBuffer: WebGLBuffer,
  indices: IndexBuffer,
  color: Color,
  mode: GLenum,
) {
  {
    gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
    gl.vertexAttribPointer(
      programInfo.attribLocations.vertexPosition,
      3, // num components
      gl.FLOAT, // type
      false, // normalize
      0, // stride
      0, // offset
    );
    gl.enableVertexAttribArray(programInfo.attribLocations.vertexPosition);
  }

  // Tell WebGL which indices to use to index the vertices
  gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indices.indexBuffer);

  // Set the shader uniforms
  gl.uniform4fv(programInfo.uniformLocations.color!, color);
  gl.uniformMatrix4fv(programInfo.uniformLocations.projectionMatrix, false, projectionMatrix);
  gl.uniformMatrix4fv(programInfo.uniformLocations.modelViewMatrix, false, modelViewMatrix);

  gl.drawElements(mode, indices.length, gl.UNSIGNED_INT, 0);
}

// Allocates and returns an ELEMENT_ARRAY_BUFFER filled with the Uint32 indices in `indices`.
// On a WebGL1 canvas the `OES_element_index_uint` extension must be loaded.
function initIndexBuffer(gl: WebGL2RenderingContext, indices: Array<number>): IndexBuffer {
  const indexBuffer = gl.createBuffer()!;
  gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);
  gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Uint32Array(indices), gl.STATIC_DRAW);
  return {
    indexBuffer: indexBuffer,
    length: indices.length,
  };
}

// Allocates and returns an ARRAY_BUFFER filled with the Float32 data in `data`.
// This type of buffer is used for vertex coordinate data and color values.
function initArrayBuffer(gl: WebGL2RenderingContext, data: Array<number>) {
  const arrayBuffer = gl.createBuffer()!;
  gl.bindBuffer(gl.ARRAY_BUFFER, arrayBuffer);
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(data), gl.STATIC_DRAW);
  return arrayBuffer;
}

// Initialize a shader program, so WebGL knows how to draw our data
function initShaderProgram(gl: WebGL2RenderingContext, vsSource: string, fsSource: string) {
  const vertexShader = loadShader(gl, gl.VERTEX_SHADER, vsSource)!;
  const fragmentShader = loadShader(gl, gl.FRAGMENT_SHADER, fsSource)!;

  const shaderProgram = gl.createProgram()!;
  gl.attachShader(shaderProgram, vertexShader);
  gl.attachShader(shaderProgram, fragmentShader);
  gl.linkProgram(shaderProgram);

  // See if creating the shader program was successful
  if (!gl.getProgramParameter(shaderProgram, gl.LINK_STATUS)) {
    throw new Error('Failed to create shader program');
  }

  return shaderProgram;
}

// creates a shader of the given type, uploads the source and compiles it.
function loadShader(gl: WebGL2RenderingContext, type: GLenum, source: string) {
  const shader = gl.createShader(type)!;
  gl.shaderSource(shader, source);
  gl.compileShader(shader);

  // See if the shader compiled successfully
  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    alert('An error occurred compiling the shaders: ' + gl.getShaderInfoLog(shader));
    gl.deleteShader(shader);
    return null;
  }

  return shader;
}

// Takes coordinates in degrees and outputs [theta, phi]
function coordinates2thetaphi(coordinate: Coordinate) {
  const phi = coordinate.lat * (Math.PI / 180);
  const theta = coordinate.long * (Math.PI / 180);
  return [theta, phi];
}

// Returns a `Coordinate` between c1 and c2.
// ratio=0.0 returns c1. ratio=1.0 returns c2.
function lerpCoordinate(c1: Coordinate, c2: Coordinate, ratio: number) {
  const lat = lerp(c1.lat, c2.lat, ratio);
  const long = lerp(c1.long, c2.long, ratio);
  return new Coordinate(lat, long);
}

// Performs linear interpolation between two floats, `x` and `y`.
function lerp(x: number, y: number, ratio: number) {
  return x + (y - x) * ratio;
}

// The shortest coordinate change from c1 to c2.
// Returns a vector representing the movement needed to go from c1 to c2 (as a `Coordinate`)
function shortestPath(c1: Coordinate, c2: Coordinate) {
  let longDiff = c2.long - c1.long;
  if (longDiff > 180) {
    longDiff -= 360;
  } else if (longDiff < -180) {
    longDiff += 360;
  }
  return new Coordinate(c2.lat - c1.lat, longDiff);
}

// smooths out a linear 0-1 transition into an accelerating and decelerating transition
function smoothTransition(x: number) {
  return 0.5 - 0.5 * Math.cos(x * Math.PI);
}

// Returns vertex positions and color values for a circle.
// `offset` is a vector of x, y and z values determining how much to offset the circle
// position from origo
function circleFanVertices(
  numEdges: number,
  radius: number,
  offset: [number, number, number],
  centerColor: Color,
  ringColor: Color,
) {
  const positions = [...offset];
  const colors = [...centerColor];
  for (let i = 0; i <= numEdges; i++) {
    const angle = (i / numEdges) * 2 * Math.PI;
    const x = offset[0] + radius * Math.cos(angle);
    const y = offset[1] + radius * Math.sin(angle);
    const z = offset[2];
    positions.push(x, y, z);
    colors.push(...ringColor);
  }
  return { positions: positions, colors: colors };
}

// Good resources:
// https://www.youtube.com/watch?v=aVwxzDHniEw - The Beauty of Bézier Curves
// https://splines.readthedocs.io/en/latest/rotation/slerp.html - slerp - spherical lerp
