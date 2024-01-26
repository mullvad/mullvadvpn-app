package net.mullvad.lib.map.shapes

import android.content.Context
import android.opengl.GLES31
import android.opengl.Matrix
import android.util.Log
import java.lang.RuntimeException
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

    private var shaderProgram: Int = 0
    private var vertexShader: Int = 0
    private var fragmentShader: Int = 0

    private var attribLocations: AttribLocations
    private var uniformLocation: UniformLocation

    data class AttribLocations(val vertexPosition: Int)

    data class UniformLocation(val color: Int, val projectionMatrix: Int, val modelViewMatrix: Int)

    data class IndexBufferWithLength(val indexBuffer: Int, val length: Int)

    private val landColor: FloatArray = floatArrayOf(0.16f, 0.302f, 0.45f, 1.0f)
    private val oceanColor: FloatArray = floatArrayOf(0.098f, 0.18f, 0.271f, 1.0f)
    private val contourColor: FloatArray = oceanColor
    private val landIndices: IndexBufferWithLength
    private val landContour: IndexBufferWithLength
    private val landPositionBuffer: Int

    private val oceanIndices: IndexBufferWithLength
    private val oceanPositionBuffer: Int

    init {

        val landPosStream = context.resources.openRawResource(R.raw.land_positions)
        val landPosByteArray = landPosStream.use { it.readBytes() }
        val landPosByteBuffer = ByteBuffer.wrap(landPosByteArray)

        Log.d("mullvad", "landTriangleIndices loaded")

        landPositionBuffer = initArrayBuffer(landPosByteBuffer)

        val landTriangleIndicesStream =
            context.resources.openRawResource(R.raw.land_triangle_indices)
        val landTriangleIndicesByteArray = landTriangleIndicesStream.use { it.readBytes() }

        // Load triangles
        val landTriangleIndicesBuffer = ByteBuffer.wrap(landTriangleIndicesByteArray)
        landIndices = initIndexBuffer(landTriangleIndicesBuffer)

        val landContourIndicesStream = context.resources.openRawResource(R.raw.land_contour_indices)
        val landContourIndicesByteArray = landContourIndicesStream.use { it.readBytes() }
        val landContourIndicesBuffer = ByteBuffer.wrap(landContourIndicesByteArray)
        landContour = initIndexBuffer(landContourIndicesBuffer)

        val oceanPosStream = context.resources.openRawResource(R.raw.ocean_positions)
        val oceanPosByteArray = oceanPosStream.use { it.readBytes() }
        val oceanPosByteBuffer = ByteBuffer.wrap(oceanPosByteArray)

        oceanPositionBuffer = initArrayBuffer(oceanPosByteBuffer)

        val oceanTriangleIndicesStream = context.resources.openRawResource(R.raw.ocean_indices)
        val oceanTriangleIndicesByteArray = oceanTriangleIndicesStream.use { it.readBytes() }

        // Load triangles
        val oceanTriangleIndicesBuffer = ByteBuffer.wrap(oceanTriangleIndicesByteArray)
        oceanIndices = initIndexBuffer(oceanTriangleIndicesBuffer)

        // create empty OpenGL ES Program
        shaderProgram = initShaderProgram(vertexShaderCode, fragmentShaderCode)

        attribLocations =
            AttribLocations(GLES31.glGetAttribLocation(shaderProgram, "aVertexPosition"))
        uniformLocation =
            UniformLocation(
                color = GLES31.glGetUniformLocation(shaderProgram, "uColor"),
                projectionMatrix = GLES31.glGetUniformLocation(shaderProgram, "uProjectionMatrix"),
                modelViewMatrix = GLES31.glGetUniformLocation(shaderProgram, "uModelViewMatrix")
            )
    }

    private fun initShaderProgram(vsSource: String, fsSource: String): Int {
        vertexShader = loadShader(GLES31.GL_VERTEX_SHADER, vsSource)
        if (vertexShader == -1) {
            throw Exception("vertexShader == -1")
        }
        fragmentShader = loadShader(GLES31.GL_FRAGMENT_SHADER, fsSource)
        if (fragmentShader == -1) {
            throw Exception("fragmentShader == -1")
        }

        val program = GLES31.glCreateProgram()
        if (program == 0) throw RuntimeException("Could not create program $program")

        // add the vertex shader to program
        GLES31.glAttachShader(program, vertexShader)

        // add the fragment shader to program
        GLES31.glAttachShader(program, fragmentShader)

        // creates OpenGL ES program executables
        GLES31.glLinkProgram(program)

        val linked = IntArray(1)
        GLES31.glGetProgramiv(program, GLES31.GL_LINK_STATUS, linked, 0)
        if (linked[0] == GLES31.GL_FALSE) {
            val infoLog = GLES31.glGetProgramInfoLog(program)
            Log.e("mullvad", "Could not link program: $infoLog")
            GLES31.glDeleteProgram(program)
            return -1
        }

        return program
    }

    private fun initArrayBuffer(dataBuffer: Buffer): Int {
        val buffer = IntArray(1)
        GLES31.glGenBuffers(1, buffer, 0)

        GLES31.glBindBuffer(GLES31.GL_ARRAY_BUFFER, buffer[0])
        GLES31.glBufferData(
            GLES31.GL_ARRAY_BUFFER,
            dataBuffer.capacity(),
            dataBuffer,
            GLES31.GL_STATIC_DRAW
        )
        return buffer[0]
    }

    private fun initIndexBuffer(dataBuffer: Buffer): IndexBufferWithLength {

        val buffer = IntArray(1)
        GLES31.glGenBuffers(1, buffer, 0)

        GLES31.glBindBuffer(GLES31.GL_ELEMENT_ARRAY_BUFFER, buffer[0])
        GLES31.glBufferData(
            GLES31.GL_ELEMENT_ARRAY_BUFFER,
            dataBuffer.capacity(),
            dataBuffer,
            GLES31.GL_STATIC_DRAW
        )

        return IndexBufferWithLength(indexBuffer = buffer[0], length = dataBuffer.capacity() / 4)
    }

    private fun loadShader(type: Int, shaderCode: String): Int {

        // create a vertex shader type (GLES31.GL_VERTEX_SHADER)
        // or a fragment shader type (GLES31.GL_FRAGMENT_SHADER)
        val shader = GLES31.glCreateShader(type)

        if (shader == 0) {
            return -1
        }
        // add the source code to the shader and compile it
        GLES31.glShaderSource(shader, shaderCode)
        GLES31.glCompileShader(shader)

        val compiled = IntArray(1)
        GLES31.glGetShaderiv(shader, GLES31.GL_COMPILE_STATUS, compiled, 0)
        if (compiled[0] == GLES31.GL_FALSE) {
            val infoLog = GLES31.glGetShaderInfoLog(shader)
            Log.e("mullvad", "Could not compile shader $type:$infoLog")
            GLES31.glDeleteShader(shader)
            return -1
        }

        return shader
    }

    fun draw(projectionMatrix: FloatArray, viewMatrix: FloatArray) {
        val globeViewMatrix = viewMatrix.clone()
        val oceanViewMatrix = viewMatrix.clone()

        // Add program to OpenGL ES environment
        GLES31.glUseProgram(shaderProgram)

        Matrix.scaleM(oceanViewMatrix, 0, 0.999f, 0.999f, 0.999f)
        drawBufferElements(
            projectionMatrix,
            oceanViewMatrix,
            oceanPositionBuffer,
            oceanIndices,
            oceanColor,
            GLES31.GL_TRIANGLES
        )

        drawBufferElements(
            projectionMatrix,
            globeViewMatrix,
            landPositionBuffer,
            landIndices,
            landColor,
            GLES31.GL_TRIANGLES,
        )
        drawBufferElements(
            projectionMatrix,
            globeViewMatrix,
            landPositionBuffer,
            landContour,
            oceanColor,
            GLES31.GL_LINES
        )
    }

    private fun drawBufferElements(
        projectionMatrix: FloatArray,
        modelViewMatrix: FloatArray,
        positionBuffer: Int,
        indexBuffer: IndexBufferWithLength,
        color: FloatArray,
        mode: Int,
    ) {
        GLES31.glBindBuffer(GLES31.GL_ARRAY_BUFFER, positionBuffer)
        GLES31.glVertexAttribPointer(
            attribLocations.vertexPosition,
            3, // Num components
            GLES31.GL_FLOAT,
            false,
            0,
            0,
        )
        GLES31.glEnableVertexAttribArray(attribLocations.vertexPosition)

        GLES31.glBindBuffer(GLES31.GL_ELEMENT_ARRAY_BUFFER, indexBuffer.indexBuffer)
        GLES31.glUniform4fv(uniformLocation.color, 1, color, 0)
        GLES31.glUniformMatrix4fv(uniformLocation.projectionMatrix, 1, false, projectionMatrix, 0)
        GLES31.glUniformMatrix4fv(uniformLocation.modelViewMatrix, 1, false, modelViewMatrix, 0)
        GLES31.glDrawElements(mode, indexBuffer.length, GLES31.GL_UNSIGNED_INT, 0)
    }
}
