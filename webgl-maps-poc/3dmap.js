// Color values for various components of the map.
const landColor = [0.16, 0.302, 0.45, 1.0];
const oceanColor = [0.098, 0.18, 0.271, 1.0];
// The color of borders between geographical entities
const contourColor = oceanColor;

// The green color of the location marker when in the secured state
const locationMarkerSecureColor = [0.267, 0.678, 0.302];
// The red color of the location marken when in the unsecured state
const locationMarkerUnsecureColor = [0.890, 0.251, 0.224];

// Zoom is distance from earths center. 1.0 is at the surface.
// These constants define the zoom levels for the connected and disconnected states.
const disconnectedZoom = 1.5;
const connectedZoom = 1.35;

// Animations longer than this time will use the out-in zoom animation.
// Shorter animations will use the direct animation.
const zoomAnimationStyleTimeBreakpoint = 1.7;
// When animating with the out-in zoom animation, set the middle
// zoom point to this times the max start or end zoom levels.
const animationZoomoutFactor = 1.2;
// Never zoom out further than this.
const maxZoomout = Math.max(disconnectedZoom, connectedZoom) * animationZoomoutFactor;

// The min and max time an animation to a new location can take.
const animationMinTime = 1.3;
const animationMaxTime = 2.5;

// The angle in degrees that the camera sees in
const angleOfView = 70;

// A geographical latitude, longitude coordinate in *degrees*.
// This class is also being abused as a 2D vector in some parts of the code.
class Coordinate {
    lat = 0.0;
    long = 0.0;
    constructor(lat, long) {
        this.lat = lat;
        this.long = long;
    }

    // When treated as a 2D vector: Get the length of said vector.
    length() {
        return Math.sqrt(this.lat * this.lat + this.long * this.long);
    }

    // When treated as a 2D vector: Scale that vector by `r`
    scale(r) {
        return new Coordinate(this.lat * r, this.long * r);
    }

    // When treated as a 2D vector: Add two vectors together returning the sum
    add(otherCoordinate) {
        return new Coordinate(this.lat + otherCoordinate.lat, this.long + otherCoordinate.long);
    }
}

// ==== DEBUG constants ==== //
const gothenburgCoordinate = new Coordinate(57.67, 11.98);
const helsinkiCoordinate = new Coordinate(60.170833, 24.9375);
const sydneyCoordinate = new Coordinate(-33.86, 151.21);
const losAngelesCoordinate = new Coordinate(34.05, -118.25);
const romeCoordinate = new Coordinate(41.893, 12.482);
const poleCoordinate1 = new Coordinate(88, -90);
const poleCoordinate2 = new Coordinate(88, 90);
const antarcticaCoordinate = new Coordinate(-85, 0);

// Class for drawing earth. Relies on a global constant `landData` being defined and
// having the structure:
//
// ```
// {
//   "positions": [<floats>],
//   "triangle_indices": [<integers>],
//   "contour_indices": [<integers>],
// }
// ```
//
// `positions`: A flat array with vertice float values for OpenGL to render.
//              each group of three floats is one vertice.
//              So `(positions[x], positions[x+1], positions[x+2])` is one vertice where
//              `x` is a multiple of 3.
//
// `triangle_indices`: The indexes in `positions` of the vertices that makes up the triangles of
//                     the earth.
//
//                     # Example
//
//                     If `[indices[0], indices[1], indices[2]] == [7, 2, 0]`,
//                     then the vertices for the first triangle to draw is:
//                     `vertice0 = [positions[7*3], positions[7*3+1], positions[7*3+2]]
//                     `vertice1 = [positions[2*3], positions[2*3+1], positions[2*3+2]]
//                     `vertice2 = [positions[0*3], positions[0*3+1], positions[0*3+2]]
//
//                     The `*3` part comes from the fact that the `indices` says which vertice,
//                     to pull from `positions`, and each vertice occupies three elements in `positions`.
//
// `contour_indices`: Same format as `triangle_indices` but denotes the indexes for drawing
//                    the contours between geographical entities/countries.
//
// It also relies on a global constant `oceanData` with the following format:
//
// ```
// {
//   positions: [<float>],
//   indices: [<integer>],
// }
// ```
//
// This is very similar to `landData`, but it denotes the vertexes and indices for the sphere that
// is drawn as the ocean.

class Globe {
    constructor(gl) {
        this.gl = gl;

        this.landVertexBuffer = initArrayBuffer(gl, landData.positions);
        this.landContourIndexBuffer = initIndexBuffer(gl, landData.contour_indices);
        this.landTriangleIndexBuffer = initIndexBuffer(gl, landData.triangle_indices);
        this.oceanVertexBuffer = initArrayBuffer(gl, oceanData.positions);
        this.oceanIndexBuffer = initIndexBuffer(gl, oceanData.indices);

        const shaderProgram = initShaderProgram(gl, Globe.vsSource, Globe.fsSource);
        this.programInfo = {
            program: shaderProgram,
            attribLocations: {
                vertexPosition: gl.getAttribLocation(shaderProgram, "aVertexPosition"),
            },
            uniformLocations: {
                color: gl.getUniformLocation(shaderProgram, "uColor"),
                projectionMatrix: gl.getUniformLocation(
                    shaderProgram,
                    "uProjectionMatrix"
                ),
                modelViewMatrix: gl.getUniformLocation(shaderProgram, "uModelViewMatrix"),
            },
        };
    }

    draw(projectionMatrix, viewMatrix) {
        const globeViewMatrix = mat4.clone(viewMatrix);

        this.gl.useProgram(this.programInfo.program);

        // Draw country contour lines
        drawBufferElements(this.gl, this.programInfo, projectionMatrix, globeViewMatrix,
            this.landVertexBuffer, this.landContourIndexBuffer, contourColor, this.gl.LINES);

        // We scale down to render the land triangles behind/under the country contour lines.
        mat4.scale(
            globeViewMatrix, // destination matrix
            globeViewMatrix, // matrix to scale
            [0.9999, 0.9999, 0.9999], // amount to scale
        );

        // Draw land triangles.
        drawBufferElements(this.gl, this.programInfo, projectionMatrix, globeViewMatrix,
            this.landVertexBuffer, this.landTriangleIndexBuffer, landColor, this.gl.TRIANGLES);

        // Draw the ocean as a sphere just beneath the land.
        drawBufferElements(this.gl, this.programInfo, projectionMatrix, globeViewMatrix,
            this.oceanVertexBuffer, this.oceanIndexBuffer, oceanColor, this.gl.TRIANGLES);
    }

    static vsSource = `
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

    static fsSource = `
    varying lowp vec4 vColor;

    void main(void) {
      gl_FragColor = vColor;
    }
  `;
}

// Class for rendering a location marker on a given coordinate on the globe.
class LocationMarker {
    constructor(gl, color) {
        this.gl = gl;

        const white = [1.0, 1.0, 1.0];
        const black = [0.0, 0.0, 0.0];
        const rings = [
            circleFanVertices(32, 0.5, [0.0, 0.0, 0.0], [...color, 0.4], [...color, 0.4]), // Semi-transparent outer
            circleFanVertices(16, 0.28, [0.0, -0.05, 0.00001], [...black, 0.55], [...black, 0.0]), // shadow
            circleFanVertices(32, 0.185, [0.0, 0.0, 0.00002], [...white, 1.0], [...white, 1.0]), // white ring
            circleFanVertices(32, 0.15, [0.0, 0.0, 0.00003], [...color, 1.0], [...color, 1.0]), // Center colored circle
        ]
        this.ringPositionCount = rings.map(r => r.positions.length);
        this.positionBuffer = initArrayBuffer(gl, rings.map(r => r.positions).flat());
        this.colorBuffer = initArrayBuffer(gl, rings.map(r => r.colors).flat());

        const shaderProgram = initShaderProgram(gl, LocationMarker.vsSource, LocationMarker.fsSource);
        this.programInfo = {
            program: shaderProgram,
            attribLocations: {
                vertexPosition: gl.getAttribLocation(shaderProgram, 'aVertexPosition'),
                vertexColor: gl.getAttribLocation(shaderProgram, 'aVertexColor'),
            },
            uniformLocations: {
                projectionMatrix: gl.getUniformLocation(shaderProgram, 'uProjectionMatrix'),
                modelViewMatrix: gl.getUniformLocation(shaderProgram, 'uModelViewMatrix'),
            },
        };
    }

    draw(projectionMatrix, viewMatrix, coordinate, size) {
        const modelViewMatrix = mat4.clone(viewMatrix);

        this.gl.useProgram(this.programInfo.program);

        var [theta, phi] = coordinates2thetaphi(coordinate);
        mat4.rotateY(modelViewMatrix, modelViewMatrix, theta);
        mat4.rotateX(modelViewMatrix, modelViewMatrix, -phi);

        mat4.scale(
            modelViewMatrix,
            modelViewMatrix,
            [size, size, 1.0]
        );
        mat4.translate(
            modelViewMatrix,
            modelViewMatrix,
            [0.0, 0.0, 1.0001]
        );

        {
            this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.positionBuffer);
            this.gl.vertexAttribPointer(
                this.programInfo.attribLocations.vertexPosition,
                3,             // num components
                this.gl.FLOAT, // type
                false,         // normalize
                0,             // stride
                0              // offset
            );
            this.gl.enableVertexAttribArray(this.programInfo.attribLocations.vertexPosition);
        }
        {
            this.gl.bindBuffer(this.gl.ARRAY_BUFFER, this.colorBuffer);
            this.gl.vertexAttribPointer(
                this.programInfo.attribLocations.vertexColor,
                4,             // num components
                this.gl.FLOAT, // type
                false,         // normalize
                0,             // stride
                0              // offset
            );
            this.gl.enableVertexAttribArray(this.programInfo.attribLocations.vertexColor);
        }

        // Set the shader uniforms
        this.gl.uniformMatrix4fv(
            this.programInfo.uniformLocations.projectionMatrix,
            false,
            projectionMatrix
        );
        this.gl.uniformMatrix4fv(
            this.programInfo.uniformLocations.modelViewMatrix,
            false,
            modelViewMatrix
        );

        let offset = 0;
        for (var i = 0; i < this.ringPositionCount.length; i++) {
            const numVertices = this.ringPositionCount[i] / 3;
            this.gl.drawArrays(this.gl.TRIANGLE_FAN, offset, numVertices);
            offset += numVertices;
        }
    }

    static vsSource = `
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

    static fsSource = `
    varying lowp vec4 vColor;

    void main(void) {
        gl_FragColor = vColor;
    }
  `;
}

// Class for computing a smooth linear interpolation from
// `startCoordinate` (2D coordinate in degrees), along `path` (2D vector),
// starting at zoom level `startZoom` and ending in `endZoom`.
// starting at time `startTime` (usually now() at the time of creating an instance),
// and animating for `duration` seconds
class SmoothLerp {
    constructor(startCoordinate, path, startZoom, endZoom, startTime, duration) {
        this.startCoordinate = startCoordinate;
        this.path = path;
        if (duration > zoomAnimationStyleTimeBreakpoint) {
            this.zoomAnimation = new SmoothZoomOutIn(startZoom, endZoom);
        } else {
            this.zoomAnimation = new SmoothZoomDirect(startZoom, endZoom);
        }
        this.startTime = startTime;
        this.duration = duration;
    }

    // Computes and returns the coordinate as well as the zoom level and the smoothened transition
    // ratio of this lerp operation.
    compute(now) {
        const animationRatio = Math.min(Math.max((now - this.startTime) / this.duration, 0.0), 1.0);
        const smoothAnimationRatio = smoothTransition(animationRatio);
        const position = this.startCoordinate.add(this.path.scale(smoothAnimationRatio));
        const zoom = this.zoomAnimation.compute(animationRatio);
        return [position, zoom, smoothAnimationRatio];
    }
}

// Zooms from startZoom to endZoom via a midpoint that is `animationZoomoutFactor` times higer up
// than max(startZoom, endZoom).
class SmoothZoomOutIn {
    constructor(startZoom, endZoom) {
        this.startZoom = startZoom;
        this.endZoom = endZoom;
        this.middleZoom = Math.min(Math.max(startZoom, endZoom) * animationZoomoutFactor, maxZoomout);
    }

    compute(animationRatio) {
        // Linear animation ratio 0-1. 0.0-0.5 means zooming out and 0.5-1.0 means zooming in
        if (animationRatio <= 0.5) {
            const smoothAnimationRatio = smoothTransition(animationRatio * 2);
            return this.startZoom + smoothAnimationRatio * (this.middleZoom - this.startZoom);
        } else {
            const smoothAnimationRatio = smoothTransition((animationRatio - 0.5) * 2);
            return this.middleZoom - smoothAnimationRatio * (this.middleZoom - this.endZoom);
        }
    }
}

// Zooms from startZoom to endZoom directly in a smooth manner.
class SmoothZoomDirect {
    constructor(startZoom, endZoom) {
        this.startZoom = startZoom;
        this.endZoom = endZoom;
    }

    compute(animationRatio) {
        const smoothAnimationRatio = smoothTransition(animationRatio);
        return this.startZoom + smoothAnimationRatio * (this.endZoom - this.startZoom);
    }
}

class Map {
    constructor(gl, startCoordinate, connectionState) {
        this.globe = new Globe(gl);
        this.locationMarkerSecure = new LocationMarker(gl, locationMarkerSecureColor);
        this.locationMarkerUnsecure = new LocationMarker(gl, locationMarkerUnsecureColor);

        this.coordinate = startCoordinate;
        this.zoom = connectionState ? connectedZoom : disconnectedZoom;
        this.connectionState = connectionState;
        // `targetCoordinate` is the same as `coordinate` when no animation is in progress.
        // This is where the location marker is drawn.
        this.targetCoordinate = startCoordinate;
        // An array of smooth lerps. Empty when no animation is in progress.
        this.animations = [];
    }

    // Move the location marker to `newCoordinate` (with state `connectionState`) and queue
    // animation to move to that coordinate.
    setLocation(newCoordinate, connectionState, now) {
        const path = shortestPath(this.coordinate, newCoordinate);
        // Compute animation time as a function of movement distance. Clamp the
        // duration range between animationMinTime and animationMaxTime
        const duration = Math.min(Math.max(path.length() / 20, animationMinTime), animationMaxTime);

        const endZoom = connectionState ? connectedZoom : disconnectedZoom;
        this.animations.push(new SmoothLerp(this.coordinate, path, this.zoom, endZoom, now, duration));

        this.connectionState = connectionState;
        this.targetCoordinate = newCoordinate;
    }

    // Render the map for the time `now`.
    draw(projectionMatrix, now) {
        this.updatePosition(now);

        const viewMatrix = mat4.create();

        // Move the camera back `this.zoom` away from the center of the globe.
        mat4.translate(
            viewMatrix, // destination matrix
            viewMatrix, // matrix to translate
            [0.0, 0.0, -this.zoom]
        );

        // Rotate the globe so the camera ends up looking down on `this.coordinate`.
        var [theta, phi] = coordinates2thetaphi(this.coordinate);
        mat4.rotateX(viewMatrix, viewMatrix, phi);
        mat4.rotateY(viewMatrix, viewMatrix, -theta);

        this.globe.draw(projectionMatrix, viewMatrix);

        // Draw the appropriate location marker depending on our connection state.
        const locationMarker = this.connectionState ? this.locationMarkerSecure : this.locationMarkerUnsecure;
        locationMarker.draw(projectionMatrix, viewMatrix, this.targetCoordinate, 0.03 * this.zoom);
    }

    // Private funciton that just updates internal animation state to match with time `now`.
    updatePosition(now) {
        if (this.animations.length === 0) {
            return;
        }

        // Compute lerp position and ratio of the newest animation
        const lastAnimation = this.animations[this.animations.length - 1];
        var [coordinate, zoom, ratio] = lastAnimation.compute(now);
        if (ratio >= 1.0) {
            // Animation is done. We can empty the animations array
            this.animations = [];
        }

        // Loop through all previous animations (that are still in progress) backwards and
        // lerp between them to compute our actual location.
        for (var i = this.animations.length - 2; i >= 0; i--) {
            const [previousPoint, previousZoom, animationRatio] = this.animations[i].compute(now);
            coordinate = lerpCoordinate(previousPoint, coordinate, ratio);
            zoom = lerp(previousZoom, zoom, ratio);
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
        this.zoom = zoom;
    }
}

main();

function main() {
    const canvas = document.querySelector("#glcanvas");
    const gl = canvas.getContext("webgl2", { antialias: true });

    if (!gl) {
        alert(
            "Unable to initialize WebGL. Your browser or machine may not support it."
        );
        return;
    }

    const map = new Map(gl, gothenburgCoordinate, true);

    // Hide triangles not facing the camera
    gl.enable(gl.CULL_FACE);
    gl.cullFace(gl.BACK);

    // Enable transparency (alpha < 1.0)
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

    // Enables using gl.UNSIGNED_INT for indexes. Allows 32 bit integer
    // indexes. Needed to have more than 2^16 vertices in one buffer.
    // Not needed on WebGL2 canvases where it's enabled by default
    // const ext = gl.getExtension('OES_element_index_uint');

    // Create a perspective matrix, a special matrix that is
    // used to simulate the distortion of perspective in a camera.
    const fieldOfView = angleOfView / 180 * Math.PI; // in radians
    const aspect = gl.canvas.clientWidth / gl.canvas.clientHeight;
    const zNear = 0.1;
    const zFar = 10;
    const projectionMatrix = mat4.create();
    mat4.perspective(projectionMatrix, fieldOfView, aspect, zNear, zFar);

    var hasSetCoordinate = 0;

    // Draw the scene repeatedly
    function render(now) {
        now *= 0.001; // convert to seconds

        if (now > 1.5 && hasSetCoordinate == 0) {
            map.setLocation(sydneyCoordinate, false, now);
            hasSetCoordinate = 1;
        } else if (now > 3 && hasSetCoordinate == 1) {
            map.setLocation(romeCoordinate, true, now);
            hasSetCoordinate = 2;
        } else if (now > 4.5 && hasSetCoordinate == 2) {
            map.setLocation(losAngelesCoordinate, false, now);
            hasSetCoordinate = 3;
        } else if (now > 8 && hasSetCoordinate == 3) {
            map.setLocation(helsinkiCoordinate, true, now);
            hasSetCoordinate = 4;
        }

        drawScene(gl, projectionMatrix, now, map);
        requestAnimationFrame(render);
    }
    requestAnimationFrame(render);
}

function drawScene(gl, projectionMatrix, now, map) {
    gl.clearColor(0.0, 0.0, 0.0, 1.0); // Clear to black, fully opaque
    gl.clearDepth(1.0); // Clear everything
    gl.enable(gl.DEPTH_TEST); // Enable depth testing
    gl.depthFunc(gl.LEQUAL); // Near things obscure far things

    // Clear the canvas before we start drawing on it.
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

    map.draw(projectionMatrix, now);
}

// Draws primitives of type `mode` (TRIANGLES, LINES etc) using vertex positions from
// `positionBuffer` at indices in `indices` with the color `color` and using the shaders in
// `programInfo`.
function drawBufferElements(gl, programInfo, projectionMatrix, modelViewMatrix, positionBuffer, indices, color, mode) {
    {
        gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
        gl.vertexAttribPointer(
            programInfo.attribLocations.vertexPosition,
            3,        // num components
            gl.FLOAT, // type
            false,    // normalize
            0,        // stride
            0         // offset
        );
        gl.enableVertexAttribArray(programInfo.attribLocations.vertexPosition);
    }

    // Tell WebGL which indices to use to index the vertices
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indices.indexBuffer);

    // Set the shader uniforms
    gl.uniform4fv(
        programInfo.uniformLocations.color,
        color,
    );
    gl.uniformMatrix4fv(
        programInfo.uniformLocations.projectionMatrix,
        false,
        projectionMatrix
    );
    gl.uniformMatrix4fv(
        programInfo.uniformLocations.modelViewMatrix,
        false,
        modelViewMatrix
    );

    gl.drawElements(mode, indices.length, gl.UNSIGNED_INT, 0);
}

// Allocates and returns an ELEMENT_ARRAY_BUFFER filled with the Uint32 indices in `indices`.
// On a WebGL1 canvas the `OES_element_index_uint` extension must be loaded.
function initIndexBuffer(gl, indices) {
    const indexBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);
    gl.bufferData(
        gl.ELEMENT_ARRAY_BUFFER,
        new Uint32Array(indices),
        gl.STATIC_DRAW
    );
    return {
        indexBuffer: indexBuffer,
        length: indices.length,
    };
}

// Allocates and returns an ARRAY_BUFFER filled with the Float32 data in `data`.
// This type of buffer is used for vertex coordinate data and color values.
function initArrayBuffer(gl, data) {
    const arrayBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, arrayBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(data), gl.STATIC_DRAW);
    return arrayBuffer;
}

// Initialize a shader program, so WebGL knows how to draw our data
function initShaderProgram(gl, vsSource, fsSource) {
    const vertexShader = loadShader(gl, gl.VERTEX_SHADER, vsSource);
    const fragmentShader = loadShader(gl, gl.FRAGMENT_SHADER, fsSource);

    const shaderProgram = gl.createProgram();
    gl.attachShader(shaderProgram, vertexShader);
    gl.attachShader(shaderProgram, fragmentShader);
    gl.linkProgram(shaderProgram);

    // See if creating the shader program was successful
    if (!gl.getProgramParameter(shaderProgram, gl.LINK_STATUS)) {
        alert(
            "Unable to initialize the shader program: " +
            gl.getProgramInfoLog(shaderProgram)
        );
        return null;
    }

    return shaderProgram;
}

// creates a shader of the given type, uploads the source and compiles it.
function loadShader(gl, type, source) {
    const shader = gl.createShader(type);
    gl.shaderSource(shader, source);
    gl.compileShader(shader);

    // See if the shader compiled successfully
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
        alert(
            "An error occurred compiling the shaders: " + gl.getShaderInfoLog(shader)
        );
        gl.deleteShader(shader);
        return null;
    }

    return shader;
}


// Takes coordinates in degrees and outputs [theta, phi]
function coordinates2thetaphi(coordinate) {
    var phi = coordinate.lat * (Math.PI / 180);
    var theta = coordinate.long * (Math.PI / 180);
    return [theta, phi];
};

// Returns a `Coordinate` between c1 and c2.
// ratio=0.0 returns c1. ratio=1.0 returns c2.
function lerpCoordinate(c1, c2, ratio) {
    const lat = lerp(c1.lat, c2.lat, ratio);
    const long = lerp(c1.long, c2.long, ratio);
    return new Coordinate(lat, long);
}

// Performs linear interpolation between two floats, `x` and `y`.
function lerp(x, y, ratio) {
    return x + (y - x) * ratio;
}

// The shortest coordinate change from c1 to c2.
// Returns a vector representing the movement needed to go from c1 to c2 (as a `Coordinate`)
function shortestPath(c1, c2) {
    var longDiff = c2.long - c1.long;
    if (longDiff > 180) {
        longDiff -= 360;
    } else if (longDiff < -180) {
        longDiff += 360;
    }
    return new Coordinate(c2.lat - c1.lat, longDiff);
}

// smooths out a linear 0-1 transition into an accelerating and decelerating transition
function smoothTransition(x) {
    return 0.5 - 0.5 * Math.cos(x * Math.PI);
}

// Returns vertex positions and color values for a circle.
// `offset` is a vector of x, y and z values determining how much to offset the circle
// position from origo
function circleFanVertices(numEdges, radius, offset, centerColor, ringColor) {
    const positions = [...offset];
    const colors = [...centerColor];
    for (var i = 0; i <= numEdges; i++) {
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