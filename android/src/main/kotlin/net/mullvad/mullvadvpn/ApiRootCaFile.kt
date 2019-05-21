package net.mullvad.mullvadvpn

import java.io.File
import java.io.FileOutputStream
import java.io.InputStream

import android.content.Context

private const val API_ROOT_CA_FILE = "api_root_ca.pem"
private const val API_ROOT_CA_PATH = "/data/data/net.mullvad.mullvadvpn/api_root_ca.pem"

class ApiRootCaFile {
    fun extract(context: Context) {
        if (!File(API_ROOT_CA_PATH).exists()) {
            extractFile(context, API_ROOT_CA_FILE, API_ROOT_CA_PATH)
        }
    }

    private fun extractFile(context: Context, asset: String, destination: String) {
        val destinationStream = FileOutputStream(destination)

        context
            .assets
            .open(asset)
            .copyTo(destinationStream)

        destinationStream.close()
    }
}
