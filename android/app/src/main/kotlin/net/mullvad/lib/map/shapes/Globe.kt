package net.mullvad.lib.map.shapes

import android.content.Context
import android.opengl.GLES20
import android.os.Build
import android.util.Log
import java.io.BufferedReader
import java.io.InputStreamReader
import java.nio.Buffer
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.nio.FloatBuffer
import net.mullvad.mullvadvpn.R

// number of coordinates per vertex in this array

private const val COORDS_PER_VERTEX = 3

class Globe(context: Context) {
    private val vertexShaderCode =
        """
attribute vec3 aVertexPosition;
uniform vec4 uColor;
uniform mat4 uModelViewMatrix;
uniform mat4 uProjectionMatrix;
varying lowp vec4 vColor;
void main(void) {
    gl_Position = uProjectionMatrix * uModelViewMatrix * vec4(aVertexPosition, 1.0);
    vColor = uColor;
}
        """
            .trimIndent()

    private val fragmentShaderCode =
        """
varying lowp vec4 vColor;
void main(void) {
    gl_FragColor = vColor;
}
        """
            .trimIndent()

    private var mProgram: Int

    init {
        val vertexShader: Int = loadShader(GLES20.GL_VERTEX_SHADER, vertexShaderCode)
        val fragmentShader: Int = loadShader(GLES20.GL_FRAGMENT_SHADER, fragmentShaderCode)

        // create empty OpenGL ES Program
        mProgram =
            GLES20.glCreateProgram().also {

                // add the vertex shader to program
                GLES20.glAttachShader(it, vertexShader)

                // add the fragment shader to program
                GLES20.glAttachShader(it, fragmentShader)

                // creates OpenGL ES program executables
                GLES20.glLinkProgram(it)
            }
        Log.d("mullvad", "AAA Globe init $mProgram")

        // Load land vertex
        val landPosStream = context.resources.openRawResource(R.raw.land_positions)
        val landPosByteArray =
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                landPosStream.readAllBytes()
            } else {
                TODO("VERSION.SDK_INT < TIRAMISU")
            }
        val landPosBuffer = ByteBuffer.wrap(landPosByteArray)
        initArrayBuffer(landPosBuffer)

//        // Load triangles
//        val landTriangleIndicesStream = context.resources.openRawResource(R.raw.land_triangle_indices)
//
//        val landTriangleIndicesByteArray =
//            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
//                landTriangleIndicesStream.readAllBytes()
//            } else {
//                TODO("VERSION.SDK_INT < TIRAMISU")
//            }
//        val buffer = ByteBuffer.wrap(landTriangleIndicesByteArray)
//        initIndexBuffer(buffer)


    }

    fun initArrayBuffer(dataBuffer: Buffer) {
        val buffer = IntArray(1)
        GLES20.glGenBuffers(1, buffer, 0)

        Log.d("mullvad", "AAA initArrayBuffer ${dataBuffer.capacity()}")
        GLES20.glBindBuffer(GLES20.GL_ARRAY_BUFFER, buffer[0])
        GLES20.glBufferData(
            GLES20.GL_ARRAY_BUFFER,
            dataBuffer.capacity(),
            dataBuffer,
            GLES20.GL_STATIC_DRAW
        )
        Log.d("mullvad", "AAA initArrayBuffer ${dataBuffer.capacity()}")
    }

    //    function initArrayBuffer(gl: WebGL2RenderingContext, data: ArrayBuffer) {
    //        const arrayBuffer = gl.createBuffer()!;
    //        gl.bindBuffer(gl.ARRAY_BUFFER, arrayBuffer);
    //        gl.bufferData(gl.ARRAY_BUFFER, data, gl.STATIC_DRAW);
    //        return arrayBuffer;
    //    }

    fun initIndexBuffer(dataBuffer: Buffer) {
        val buffer = IntArray(1)
        GLES20.glGenBuffers(1, buffer, 0)

        GLES20.glBindBuffer(GLES20.GL_ELEMENT_ARRAY_BUFFER, buffer[0])
        GLES20.glBufferData(
            GLES20.GL_ELEMENT_ARRAY_BUFFER,
            dataBuffer.capacity(),
            dataBuffer,
            GLES20.GL_STATIC_DRAW
        )
    }
//    function initIndexBuffer(gl: WebGL2RenderingContext, indices: ArrayBuffer): IndexBuffer {
//        const indexBuffer = gl.createBuffer()!;
//        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indexBuffer);
//        gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, indices, gl.STATIC_DRAW);
//        return {
//            indexBuffer: indexBuffer,
//            // Values are 32 bit, i.e. 4 bytes per value
//            length: indices.byteLength / 4,
//        };
//    }

    fun loadShader(type: Int, shaderCode: String): Int {

        // create a vertex shader type (GLES20.GL_VERTEX_SHADER)
        // or a fragment shader type (GLES20.GL_FRAGMENT_SHADER)
        return GLES20.glCreateShader(type).also { shader ->

            // add the source code to the shader and compile it
            GLES20.glShaderSource(shader, shaderCode)
            GLES20.glCompileShader(shader)
        }
    }

    fun draw() {
//        // Add program to OpenGL ES environment
//        GLES20.glUseProgram(mProgram)
//
//        // get handle to vertex shader's vPosition member
//        positionHandle =
//            GLES20.glGetAttribLocation(mProgram, "vPosition").also {
//
//                // Enable a handle to the triangle vertices
//                GLES20.glEnableVertexAttribArray(it)
//
//                // Prepare the triangle coordinate data
//                GLES20.glVertexAttribPointer(
//                    it,
//                    COORDS_PER_VERTEX,
//                    GLES20.GL_FLOAT,
//                    false,
//                    vertexStride,
//                    vertexBuffer
//                )
//
//                // get handle to fragment shader's vColor member
//                mColorHandle =
//                    GLES20.glGetUniformLocation(mProgram, "vColor").also { colorHandle ->
//
//                        // Set color for drawing the triangle
//                        GLES20.glUniform4fv(colorHandle, 1, color, 0)
//                    }
//
//                // Draw the triangle
//                GLES20.glDrawArrays(GLES20.GL_TRIANGLES, 0, vertexCount)
//
//                // Disable vertex array
//                GLES20.glDisableVertexAttribArray(it)
//            }
    }
//    // Draws primitives of type `mode` (TRIANGLES, LINES etc) using vertex positions from
//// `positionBuffer` at indices in `indices` with the color `color` and using the shaders in
//// `programInfo`.
//    function drawBufferElements(
//    gl: WebGL2RenderingContext,
//    programInfo: ProgramInfo,
//    projectionMatrix: mat4,
//    modelViewMatrix: mat4,
//    positionBuffer: WebGLBuffer,
//    indices: IndexBuffer,
//    color: Color,
//    mode: GLenum,
//    ) {
//        {
//            gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
//            gl.vertexAttribPointer(
//                programInfo.attribLocations.vertexPosition,
//                3, // num components
//                gl.FLOAT, // type
//                false, // normalize
//                0, // stride
//                0, // offset
//            );
//            gl.enableVertexAttribArray(programInfo.attribLocations.vertexPosition);
//        }
//
//        // Tell WebGL which indices to use to index the vertices
//        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, indices.indexBuffer);
//
//        // Set the shader uniforms
//        gl.uniform4fv(programInfo.uniformLocations.color!, color);
//        gl.uniformMatrix4fv(programInfo.uniformLocations.projectionMatrix, false, projectionMatrix);
//        gl.uniformMatrix4fv(programInfo.uniformLocations.modelViewMatrix, false, modelViewMatrix);
//
//        gl.drawElements(mode, indices.length, gl.UNSIGNED_INT, 0);
//    }


}
