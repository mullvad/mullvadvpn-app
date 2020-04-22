package net.mullvad.mullvadvpn.service

import android.util.Log
import java.io.File

class FileMigrator(val oldDirectory: File, val newDirectory: File) {
    fun migrate(fileName: String) {
        try {
            val oldPath = File(oldDirectory, fileName)

            if (oldPath.exists()) {
                oldPath.renameTo(File(newDirectory, fileName))
            }
        } catch (exception: Exception) {
            Log.w("mullvad", "Failed to migrate $fileName from $oldDirectory to $newDirectory")
        }
    }
}
