package net.mullvad.lib.map.shapes

import android.opengl.GLES31
import android.util.Log
import java.nio.Buffer
import kotlin.RuntimeException

object GLHelper {

    fun initShaderProgram(vsSource: String, fsSource: String): Int {
        val vertexShader = loadShader(GLES31.GL_VERTEX_SHADER, vsSource)
        if (vertexShader == -1) {
            throw RuntimeException("vertexShader == -1")
        }
        val fragmentShader = loadShader(GLES31.GL_FRAGMENT_SHADER, fsSource)
        if (fragmentShader == -1) {
            throw RuntimeException("fragmentShader == -1")
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

        Log.d("mullvad", "CREATED PROGRAM $program")

        return program
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

    fun initArrayBuffer(dataBuffer: Buffer): Int {
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

    data class IndexBufferWithLength(val indexBuffer: Int, val length: Int)

    fun initIndexBuffer(dataBuffer: Buffer): IndexBufferWithLength {

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
}
