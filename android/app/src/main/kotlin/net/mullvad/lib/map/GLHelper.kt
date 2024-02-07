package net.mullvad.lib.map

import android.opengl.GLES20
import android.util.Log
import java.lang.IllegalStateException
import java.nio.Buffer
import java.nio.ByteBuffer
import java.nio.FloatBuffer

object GLHelper {

    fun initShaderProgram(vsSource: String, fsSource: String): Int {
        val vertexShader = loadShader(GLES20.GL_VERTEX_SHADER, vsSource)
        require(vertexShader != -1) { "vertexShader == -1" }

        val fragmentShader = loadShader(GLES20.GL_FRAGMENT_SHADER, fsSource)
        require(fragmentShader != -1) { "fragmentShader == -1" }

        val program = GLES20.glCreateProgram()
        check(program != 0) { "Could not create program"}

        // add the vertex shader to program
        GLES20.glAttachShader(program, vertexShader)

        // add the fragment shader to program
        GLES20.glAttachShader(program, fragmentShader)

        // creates OpenGL ES program executables
        GLES20.glLinkProgram(program)

        val linked = IntArray(1)
        GLES20.glGetProgramiv(program, GLES20.GL_LINK_STATUS, linked, 0)
        if (linked[0] == GLES20.GL_FALSE) {
            val infoLog = GLES20.glGetProgramInfoLog(program)
            Log.e("mullvad", "Could not link program: $infoLog")
            GLES20.glDeleteProgram(program)
            error("Could not link program with vsSource: $vsSource and fsSource: $fsSource")
        }

        return program
    }

    private fun loadShader(type: Int, shaderCode: String): Int {
        // create a vertex shader type (GLES20.GL_VERTEX_SHADER)
        // or a fragment shader type (GLES20.GL_FRAGMENT_SHADER)
        val shader = GLES20.glCreateShader(type)

        require(shader != 0) { "Unable to create shader" }

        // add the source code to the shader and compile it
        GLES20.glShaderSource(shader, shaderCode)
        GLES20.glCompileShader(shader)

        val compiled = IntArray(1)
        GLES20.glGetShaderiv(shader, GLES20.GL_COMPILE_STATUS, compiled, 0)
        if (compiled[0] == GLES20.GL_FALSE) {
            val infoLog = GLES20.glGetShaderInfoLog(shader)
            Log.e("mullvad", "Could not compile shader $type:$infoLog")
            GLES20.glDeleteShader(shader)
            throw IllegalArgumentException("Could not compile shader with shaderCode: $shaderCode")
        }

        return shader
    }

    fun initArrayBuffer(buffer: ByteBuffer) = initArrayBuffer(buffer, 1)

    fun initArrayBuffer(buffer: FloatBuffer) = initArrayBuffer(buffer, Float.SIZE_BYTES)

    private fun initArrayBuffer(dataBuffer: Buffer, unitSizeInBytes: Int = 1): Int {
        val buffer = IntArray(1)
        GLES20.glGenBuffers(1, buffer, 0)

        GLES20.glBindBuffer(GLES20.GL_ARRAY_BUFFER, buffer[0])
        GLES20.glBufferData(
            GLES20.GL_ARRAY_BUFFER,
            dataBuffer.capacity() * unitSizeInBytes,
            dataBuffer,
            GLES20.GL_STATIC_DRAW
        )
        return buffer[0]
    }

    data class IndexBufferWithLength(val indexBuffer: Int, val length: Int)

    fun initIndexBuffer(dataBuffer: Buffer): IndexBufferWithLength {

        val buffer = IntArray(1)
        GLES20.glGenBuffers(1, buffer, 0)

        GLES20.glBindBuffer(GLES20.GL_ELEMENT_ARRAY_BUFFER, buffer[0])
        GLES20.glBufferData(
            GLES20.GL_ELEMENT_ARRAY_BUFFER,
            dataBuffer.capacity(),
            dataBuffer,
            GLES20.GL_STATIC_DRAW
        )
        return IndexBufferWithLength(indexBuffer = buffer[0], length = dataBuffer.capacity() / 4)
    }
}
