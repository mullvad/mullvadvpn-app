package net.mullvad.mullvadvpn.lib.map.internal.shapes

import android.content.res.Resources
import android.opengl.GLES20
import android.opengl.Matrix
import java.nio.ByteBuffer
import net.mullvad.mullvadvpn.lib.map.R
import net.mullvad.mullvadvpn.lib.map.data.GlobeColors
import net.mullvad.mullvadvpn.lib.map.internal.GLIndexBuffer
import net.mullvad.mullvadvpn.lib.map.internal.VERTEX_COMPONENT_SIZE
import net.mullvad.mullvadvpn.lib.map.internal.initGLArrayBuffer
import net.mullvad.mullvadvpn.lib.map.internal.initGLIndexBuffer
import net.mullvad.mullvadvpn.lib.map.internal.initShaderProgram

internal class Globe(resources: Resources) {
    private val shaderProgram: Int

    private val attribLocations: AttribLocations
    private val uniformLocation: UniformLocation

    private val landVertexBuffer: Int
    private val landIndices: GLIndexBuffer
    private val landContourIndices: GLIndexBuffer

    private val oceanVertexBuffer: Int
    private val oceanIndices: GLIndexBuffer

    init {
        val landVertByteBuffer = resources.loadRawByteBuffer(R.raw.land_positions)
        landVertexBuffer = initGLArrayBuffer(landVertByteBuffer)
        val landTriangleIndicesBuffer = resources.loadRawByteBuffer(R.raw.land_triangle_indices)
        landIndices = initGLIndexBuffer(landTriangleIndicesBuffer)
        val landContourIndicesBuffer = resources.loadRawByteBuffer(R.raw.land_contour_indices)
        landContourIndices = initGLIndexBuffer(landContourIndicesBuffer)

        val oceanVertByteBuffer = resources.loadRawByteBuffer(R.raw.ocean_positions)
        oceanVertexBuffer = initGLArrayBuffer(oceanVertByteBuffer)
        val oceanTriangleIndicesBuffer = resources.loadRawByteBuffer(R.raw.ocean_indices)
        oceanIndices = initGLIndexBuffer(oceanTriangleIndicesBuffer)

        // create empty OpenGL ES Program
        shaderProgram = initShaderProgram(vertexShaderCode, fragmentShaderCode)

        attribLocations =
            AttribLocations(GLES20.glGetAttribLocation(shaderProgram, "aVertexPosition"))
        uniformLocation =
            UniformLocation(
                color = GLES20.glGetUniformLocation(shaderProgram, "uColor"),
                projectionMatrix = GLES20.glGetUniformLocation(shaderProgram, "uProjectionMatrix"),
                modelViewMatrix = GLES20.glGetUniformLocation(shaderProgram, "uModelViewMatrix"),
            )
    }

    private fun Resources.loadRawByteBuffer(id: Int): ByteBuffer {
        val inputStream = openRawResource(id)
        val byteArray = inputStream.use { it.readBytes() }
        return ByteBuffer.wrap(byteArray)
    }

    fun draw(
        projectionMatrix: FloatArray,
        viewMatrix: FloatArray,
        colors: GlobeColors,
        contourWidth: Float = 3f,
    ) {
        val globeViewMatrix = viewMatrix.copyOf()

        // Add program to OpenGL ES environment
        GLES20.glUseProgram(shaderProgram)

        // Set thickness of contour lines
        GLES20.glLineWidth(contourWidth)
        drawBufferElements(
            projectionMatrix,
            globeViewMatrix,
            landVertexBuffer,
            landContourIndices,
            colors.contourColorArray,
            GLES20.GL_LINE_STRIP,
        )

        // Scale the globe to avoid z-fighting
        Matrix.scaleM(
            globeViewMatrix,
            0,
            LAND_OCEAN_SCALE_FACTOR,
            LAND_OCEAN_SCALE_FACTOR,
            LAND_OCEAN_SCALE_FACTOR,
        )

        // Draw land
        drawBufferElements(
            projectionMatrix,
            globeViewMatrix,
            landVertexBuffer,
            landIndices,
            colors.landColorArray,
            GLES20.GL_TRIANGLES,
        )

        // Draw ocean
        drawBufferElements(
            projectionMatrix,
            globeViewMatrix,
            oceanVertexBuffer,
            oceanIndices,
            colors.oceanColorArray,
            GLES20.GL_TRIANGLES,
        )
    }

    private fun drawBufferElements(
        projectionMatrix: FloatArray,
        modelViewMatrix: FloatArray,
        positionBuffer: Int,
        indexBuffer: GLIndexBuffer,
        color: FloatArray,
        mode: Int,
    ) {
        GLES20.glBindBuffer(GLES20.GL_ARRAY_BUFFER, positionBuffer)
        GLES20.glVertexAttribPointer(
            attribLocations.vertexPosition,
            VERTEX_COMPONENT_SIZE,
            GLES20.GL_FLOAT,
            false,
            0,
            0,
        )
        GLES20.glEnableVertexAttribArray(attribLocations.vertexPosition)

        GLES20.glBindBuffer(GLES20.GL_ELEMENT_ARRAY_BUFFER, indexBuffer.indexBuffer)
        GLES20.glUniform4fv(uniformLocation.color, 1, color, 0)
        GLES20.glUniformMatrix4fv(uniformLocation.projectionMatrix, 1, false, projectionMatrix, 0)
        GLES20.glUniformMatrix4fv(uniformLocation.modelViewMatrix, 1, false, modelViewMatrix, 0)
        GLES20.glDrawElements(mode, indexBuffer.length, GLES20.GL_UNSIGNED_INT, 0)
        GLES20.glDisableVertexAttribArray(attribLocations.vertexPosition)
    }

    private data class AttribLocations(val vertexPosition: Int)

    private data class UniformLocation(
        val color: Int,
        val projectionMatrix: Int,
        val modelViewMatrix: Int,
    )

    companion object {
        private const val LAND_OCEAN_SCALE_FACTOR = 0.99999f

        // Vertex, and fragment shader code is taken from Mullvad Desktop 3dmap.ts
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
    }
}
