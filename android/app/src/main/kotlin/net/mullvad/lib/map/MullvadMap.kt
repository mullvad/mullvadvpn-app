package net.mullvad.lib.map

import android.content.Context
import android.content.res.Resources
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import net.mullvad.mullvadvpn.R
import java.io.BufferedReader
import java.io.IOException
import java.io.InputStreamReader

@Composable
fun MullvadMap() {
    val context = LocalContext.current

    val fragmentShader = remember { context.readTextFileFromResource(R.raw.fragment_shader) }

    val vertexShader = remember { context.readTextFileFromResource(R.raw.vertex_shader) }

    val shaderRenderer = remember {
        ShaderRenderer().apply { setShaders(fragmentShader, vertexShader, source = "dummy") }
    }

    GLShader(
        renderer = shaderRenderer,
        modifier = Modifier.fillMaxWidth().fillMaxHeight(),
    )
}

private fun Context.readTextFileFromResource(
    resourceId: Int
): String {
    val body = StringBuilder()
    try {
        val inputStream = resources.openRawResource(resourceId)
        val inputStreamReader = InputStreamReader(inputStream)
        val bufferedReader = BufferedReader(inputStreamReader)
        var nextLine: String?
        while (bufferedReader.readLine().also { nextLine = it } != null) {
            body.append(nextLine)
            body.append('\n')
        }
    } catch (e: IOException) {
        throw RuntimeException(
            "Could not open resource: $resourceId", e
        )
    } catch (nfe: Resources.NotFoundException) {
        throw RuntimeException("Resource not found: $resourceId", nfe)
    }
    return body.toString()
}
