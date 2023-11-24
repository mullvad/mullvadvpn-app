package net.mullvad.lib.map

import android.graphics.Bitmap
import android.opengl.GLES31.*
import android.opengl.GLSurfaceView
import android.os.Trace
import android.util.Log
import androidx.palette.graphics.Palette
import androidx.palette.graphics.Target
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.util.concurrent.atomic.AtomicBoolean
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10
import kotlin.math.roundToInt

open class ShaderRenderer : GLSurfaceView.Renderer {

    private val positionComponentCount = 2

    private val quadVertices by lazy {
        floatArrayOf(
            -1f, 1f,
            1f, 1f,
            -1f, -1f,
            1f, -1f
        )
    }

    private var surfaceHeight = 0f
    private var surfaceWidth = 0f

    private val bytesPerFloat = 4

    private val verticesData by lazy {
        ByteBuffer.allocateDirect(quadVertices.size * bytesPerFloat)
            .order(ByteOrder.nativeOrder()).asFloatBuffer().also {
                it.put(quadVertices)
            }
    }

    private var snapshotBuffer = initializeSnapshotBuffer(0, 0)

    private fun initializeSnapshotBuffer(width: Int, height: Int) = ByteBuffer.allocateDirect(
        width *
                height *
                bytesPerFloat
    ).order(ByteOrder.nativeOrder())

    override fun onSurfaceCreated(gl: GL10?, config: EGLConfig?) {
        glClearColor(0f, 0f, 0f, 1f)
        glDisable(GL10.GL_DITHER)
        glHint(GL10.GL_PERSPECTIVE_CORRECTION_HINT, GL10.GL_FASTEST)
    }

    private val isProgramChanged = AtomicBoolean(false)

    private var programId: Int? = null

    private lateinit var fragmentShader: String
    private lateinit var vertexShader: String
    private lateinit var eventSource: String

    fun setShaders(fragmentShader: String, vertexShader: String, source: String) {
        this.fragmentShader = fragmentShader
        this.vertexShader = vertexShader
        this.eventSource = source
        shouldPlay.compareAndSet(false, true)
        isProgramChanged.compareAndSet(false, true)
    }

    private fun setupProgram() {
        programId?.let { glDeleteProgram(it) }

        programId = glCreateProgram().also { newProgramId ->
            if (programId == 0) {
                Log.d("mullvad", "AAA Could not create new program")
                return
            }

            val fragShader = createAndVerifyShader(fragmentShader, GL_FRAGMENT_SHADER)
            val vertShader = createAndVerifyShader(vertexShader, GL_VERTEX_SHADER)

            glAttachShader(newProgramId, vertShader)
            glAttachShader(newProgramId, fragShader)

            glLinkProgram(newProgramId)

            val linkStatus = IntArray(1)
            glGetProgramiv(newProgramId, GL_LINK_STATUS, linkStatus, 0)

            if (linkStatus[0] == 0) {
                glDeleteProgram(newProgramId)
                Log.d("mullvad", "AAA Linking of program failed. ${glGetProgramInfoLog(newProgramId)}")
                return
            }

            if (validateProgram(newProgramId)) {
                positionAttributeLocation = glGetAttribLocation(newProgramId, "a_Position")
                resolutionUniformLocation = glGetUniformLocation(newProgramId, "u_resolution")
                timeUniformLocation = glGetUniformLocation(newProgramId, "u_time")
            } else {
                Log.d("mullvad", "AAA Validating of program failed.");
                return
            }

            verticesData.position(0)

            positionAttributeLocation?.let { attribLocation ->
                glVertexAttribPointer(
                    attribLocation,
                    positionComponentCount,
                    GL_FLOAT,
                    false,
                    0,
                    verticesData
                )
            }

            glDetachShader(newProgramId, vertShader)
            glDetachShader(newProgramId, fragShader)
            glDeleteShader(vertShader)
            glDeleteShader(fragShader)
        }
    }

    private var positionAttributeLocation: Int? = null
    private var resolutionUniformLocation: Int? = null
    private var timeUniformLocation: Int? = null

    override fun onSurfaceChanged(gl: GL10?, width: Int, height: Int) {
        glViewport(0, 0, width, height)
        snapshotBuffer = initializeSnapshotBuffer(width, height)
        surfaceWidth = width.toFloat()
        surfaceHeight = height.toFloat()
        frameCount = 0f


    }

    private var frameCount = 0f

    override fun onDrawFrame(gl: GL10?) {

        /**
        if (shouldPlay.get()) {
            Trace.beginSection(eventSource)
            glDisable(GL10.GL_DITHER)
            glClear(GL10.GL_COLOR_BUFFER_BIT)


            if (isProgramChanged.getAndSet(false)) {
                setupProgram()
            } else {
                programId?.let {
                    glUseProgram(it)
                } ?: return
            }

            positionAttributeLocation?.let {
                glEnableVertexAttribArray(it)
            } ?: return


            resolutionUniformLocation?.let {
                glUniform2f(it, surfaceWidth, surfaceHeight)
            }

            timeUniformLocation?.let {
                glUniform1f(it, frameCount)
            }

            glDrawArrays(GL_TRIANGLE_STRIP, 0, 4)

            positionAttributeLocation?.let {
                glDisableVertexAttribArray(it)
            } ?: return

            getPaletteCallback?.let { callback ->
                if (surfaceWidth != 0f && surfaceHeight != 0f) {
                    getCurrentBitmap()?.let { bitmap ->
                        Palette.Builder(bitmap)
                            .maximumColorCount(6)
                            .addTarget(Target.VIBRANT)
                            .generate().let { palette ->
                                callback(palette)
                                getPaletteCallback = null
                                bitmap.recycle()
                            }
                    }
                }
            }

//            glFinish()

            if (frameCount > 30) {
                frameCount = 0f
            }

            frameCount += 0.01f

            Trace.endSection()
        }
        */
    }

    private fun getCurrentBitmap(): Bitmap? {
        val maxWidth = surfaceWidth.roundToInt()
        val maxHeight = surfaceHeight.roundToInt()

        val quarterWidth = maxWidth / 6
        val quarterHeight = maxHeight / 6

        val halfWidth = quarterWidth * 2
        val halfHeight = quarterHeight * 2

        initializeSnapshotBuffer(
            halfWidth * 2,
            halfHeight * 2,
        )

        glReadPixels(
            halfWidth,
            halfHeight,
            halfWidth * 2,
            halfHeight * 2,
            GL_RGBA,
            GL_UNSIGNED_BYTE,
            snapshotBuffer
        )

        val bitmap = Bitmap.createBitmap(
            24,
            24,
            Bitmap.Config.ARGB_8888
        )

        bitmap.copyPixelsFromBuffer(snapshotBuffer)
        return bitmap
    }

    private fun validateProgram(programObjectId: Int): Boolean {
        glValidateProgram(programObjectId)
        val validateStatus = IntArray(1)
        glGetProgramiv(programObjectId, GL_VALIDATE_STATUS, validateStatus, 0)

//        Timber.tag("Results of validating").v(
//            "${validateStatus[0]} \n  Log : ${
//                glGetProgramInfoLog(
//                    programObjectId
//                )
//            } \n".trimIndent()
//        )

        return validateStatus[0] != 0
    }

    private var getPaletteCallback: ((Palette) -> Unit)? = null

    fun setPaletteCallback(callback: (Palette) -> Unit) {
        getPaletteCallback = callback
    }

    private val shouldPlay = AtomicBoolean(false)

    fun onResume() {
        shouldPlay.compareAndSet(false, ::fragmentShader.isInitialized)
    }

    fun onPause() {
        shouldPlay.compareAndSet(true, false)
    }

}
