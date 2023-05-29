package net.mullvad.mullvadvpn.applist

import android.content.pm.PackageManager
import android.graphics.Bitmap
import android.os.Looper
import androidx.annotation.WorkerThread
import androidx.collection.LruCache
import androidx.core.graphics.drawable.toBitmap

class ApplicationsIconManager(private val packageManager: PackageManager) {
    private val iconsCache = LruCache<String, Bitmap>(500)

    @WorkerThread
    @Throws(PackageManager.NameNotFoundException::class)
    fun getAppIcon(packageName: String): Bitmap {
        check(!Looper.getMainLooper().isCurrentThread) { "Should not be called from MainThread" }
        iconsCache.get(packageName)?.let {
            return it
        }
        return packageManager.getApplicationIcon(packageName).toBitmap().also {
            iconsCache.put(packageName, it)
        }
    }

    fun dispose() {
        iconsCache.evictAll()
    }
}
