package net.mullvad.lib.map

import android.content.Context
import android.opengl.GLSurfaceView
import android.util.AttributeSet
import android.util.Log

class ShaderGLSurfaceView
@JvmOverloads
constructor(
    context: Context,
    attrs: AttributeSet? = null,
) : GLSurfaceView(context, attrs) {

    init {

        // Create an OpenGL ES 2.0 context
        setEGLContextClientVersion(2)

        preserveEGLContextOnPause = true
    }

    private var hasSetShader = false

    fun setShaderRenderer(renderer: Renderer) {

        if (hasSetShader.not()) setRenderer(renderer)

        hasSetShader = true
    }

    override fun onResume() {
        super.onResume()
        Log.d("mullvad", "AAA ShaderGLSurfaceView onResume")
    }

    override fun onPause() {
        super.onPause()
        Log.d("mullvad", "AAA ShaderGLSurfaceView onPause")
    }

    override fun onDetachedFromWindow() {
        super.onDetachedFromWindow()
        Log.d("mullvad", "AAA ShaderGLSurfaceView onDetachedFromWindow")
    }
}
