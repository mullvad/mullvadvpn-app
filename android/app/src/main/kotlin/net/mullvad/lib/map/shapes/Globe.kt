package net.mullvad.lib.map.shapes

import android.content.Context
import android.opengl.GLES20
import android.os.Build
import android.util.Log
import java.nio.Buffer
import java.nio.ByteBuffer
import net.mullvad.mullvadvpn.R

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

    private val shaderProgram: Int
    private val vertexShader: Int
    private val fragmentShader: Int

    private val attribLocations: AttribLocations
    private val uniformLocation: UniformLocation

    data class AttribLocations(val vertexPosition: Int)

    data class UniformLocation(val color: Int, val projectionMatrix: Int, val modelViewMatrix: Int)

    data class IndexBuffer(val indexBuffer: Int, val length: Int)

    val landColor: FloatArray = floatArrayOf(0.16f, 0.302f, 0.45f, 1.0f)
//    val oceanColor: FloatArray = floatArrayOf(0.098f, 0.18f, 0.271f, 1.0f)
    val landIndices: IndexBuffer
    val landPosBuffer: Int

    init {
        vertexShader = loadShader(GLES20.GL_VERTEX_SHADER, vertexShaderCode)
        fragmentShader = loadShader(GLES20.GL_FRAGMENT_SHADER, fragmentShaderCode)

        // create empty OpenGL ES Program
        shaderProgram =
            GLES20.glCreateProgram().also {

                // add the vertex shader to program
                GLES20.glAttachShader(it, vertexShader)

                // add the fragment shader to program
                GLES20.glAttachShader(it, fragmentShader)

                // creates OpenGL ES program executables
                GLES20.glLinkProgram(it)
            }
        Log.d("mullvad", "AAA Globe init $shaderProgram")

        val buffer = IntArray(2)
        GLES20.glGenBuffers(2, buffer, 0)

        // Load land vertex
        val landPosStream = context.resources.openRawResource(R.raw.land_positions)
        val landPosByteArray =
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                landPosStream.readAllBytes()
            } else {
                TODO("VERSION.SDK_INT < TIRAMISU")
            }

        if(landPosByteArray == null) {
            throw Exception("landPosByteArray null!")
        } else {
            Log.d("mullvad", "landPosByteArray not null")
        }

        val landPosBufferData = ByteBuffer.wrap(landPosByteArray).asFloatBuffer()
        if(landPosBufferData == null) {
            throw Exception("landPosByteArray null!")
        } else {
            Log.d("mullvad", "bufferData not null")
        }
        landPosBuffer = buffer[0]
        initArrayBuffer(buffer[0], landPosBufferData)

        Log.d("mullvad", "landPosBufferData loaded")
        // Load triangles
        val landTriangleIndicesStream =
            context.resources.openRawResource(R.raw.land_triangle_indices)

        val landTriangleIndicesByteArray =
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                landTriangleIndicesStream.readAllBytes()
            } else {
                TODO("VERSION.SDK_INT < TIRAMISU")
            }
        val landTriangleIndicesBuffer = ByteBuffer.wrap(landTriangleIndicesByteArray)
        landIndices = initIndexBuffer(buffer[1], landTriangleIndicesBuffer)

        Log.d("mullvad", "landTriangleIndices loaded")
        attribLocations =
            AttribLocations(GLES20.glGetAttribLocation(shaderProgram, "aVertexPosition"))
        uniformLocation =
            UniformLocation(
                color = GLES20.glGetUniformLocation(shaderProgram, "uColor"),
                projectionMatrix = GLES20.glGetUniformLocation(shaderProgram, "uProjectionMatrix"),
                modelViewMatrix = GLES20.glGetUniformLocation(shaderProgram, "uModelViewMatrix")
            )
    }

    fun initArrayBuffer(bufferIndex: Int, dataBuffer: Buffer) {
        Log.d("mullvad", "AAA initArrayBuffer ${dataBuffer.capacity()}")
        GLES20.glBindBuffer(GLES20.GL_ARRAY_BUFFER, bufferIndex)
        Log.d("mullvad", "AAA initArrayBuffer times 4: ${dataBuffer.capacity()*4}")
        GLES20.glBufferData(
            GLES20.GL_ARRAY_BUFFER,
            dataBuffer.capacity() * 4,
            dataBuffer,
            GLES20.GL_STATIC_DRAW
        )
        Log.d("mullvad", "Post")
    }

    fun initIndexBuffer(bufferIndex: Int, dataBuffer: Buffer): IndexBuffer {
        GLES20.glBindBuffer(GLES20.GL_ELEMENT_ARRAY_BUFFER, bufferIndex)
        GLES20.glBufferData(
            GLES20.GL_ELEMENT_ARRAY_BUFFER,
            dataBuffer.capacity(),
            dataBuffer,
            GLES20.GL_STATIC_DRAW
        )

        return IndexBuffer(indexBuffer = bufferIndex, length = dataBuffer.capacity() / 4)
    }

    fun loadShader(type: Int, shaderCode: String): Int {

        // create a vertex shader type (GLES20.GL_VERTEX_SHADER)
        // or a fragment shader type (GLES20.GL_FRAGMENT_SHADER)
        return GLES20.glCreateShader(type).also { shader ->

            // add the source code to the shader and compile it
            GLES20.glShaderSource(shader, shaderCode)
            GLES20.glCompileShader(shader)
        }
    }

    fun draw(projectionMatrix: FloatArray, viewMatrix: FloatArray) {
        val globeViewMatrix = viewMatrix.clone()

        // Add program to OpenGL ES environment
        GLES20.glUseProgram(shaderProgram)

        drawBufferElements(
            GLES20.GL_TRIANGLES,
            landPosBuffer,
            landIndices,
            landColor,
            projectionMatrix,
            globeViewMatrix
        )
    }

    fun drawBufferElements(
        mode: Int,
        positionBuffer: Int,
        indexBuffer: IndexBuffer,
        color: FloatArray,
        projectionMatrix: FloatArray,
        modelViewMatrix: FloatArray,
    ) {
        GLES20.glBindBuffer(GLES20.GL_ARRAY_BUFFER, positionBuffer)
        GLES20.glVertexAttribPointer(
            attribLocations.vertexPosition,
            3, // Num components
            GLES20.GL_FLOAT,
            false,
            0,
            0,
        )
        GLES20.glEnableVertexAttribArray(attribLocations.vertexPosition)

        GLES20.glBindBuffer(GLES20.GL_ELEMENT_ARRAY_BUFFER, indexBuffer.indexBuffer)
        GLES20.glUniform4fv(uniformLocation.color, color.size, color, 0)
        GLES20.glUniformMatrix4fv(
            uniformLocation.projectionMatrix,
            projectionMatrix.size,
            false,
            projectionMatrix,
            0
        )
        GLES20.glUniformMatrix4fv(
            uniformLocation.modelViewMatrix,
            modelViewMatrix.size,
            false,
            modelViewMatrix,
            0
        )
        GLES20.glDrawElements(mode, indexBuffer.length, GLES20.GL_UNSIGNED_INT, 0)
    }
}
