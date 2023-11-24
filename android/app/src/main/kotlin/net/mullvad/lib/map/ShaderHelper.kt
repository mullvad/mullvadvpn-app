package net.mullvad.lib.map

import android.opengl.GLES31
import android.util.Log

internal fun createAndVerifyShader(shaderCode: String, shaderType: Int): Int {
    val shaderId = GLES31.glCreateShader(shaderType)
    if (shaderId == 0) {
        Log.d("mullvad", "AAA Create Shader failed")
    }

    GLES31.glShaderSource(shaderId, shaderCode)
    GLES31.glCompileShader(shaderId)

    val compileStatusArray = IntArray(1)
    GLES31.glGetShaderiv(shaderId, GLES31.GL_COMPILE_STATUS, compileStatusArray, 0)
    Log.d("mullvad", "AAA $shaderCode \n : ${GLES31.glGetShaderInfoLog(shaderId)}")

    if (compileStatusArray.first() == 0) {
        GLES31.glDeleteShader(shaderId)
        return 0
    }

    return shaderId
}
