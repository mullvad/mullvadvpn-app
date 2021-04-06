package net.mullvad.mullvadvpn.applist

import android.content.pm.PackageManager
import android.graphics.drawable.Drawable
import android.os.Looper
import androidx.annotation.WorkerThread
import androidx.collection.LruCache

class ApplicationsIconManager(private val packageManager: PackageManager) {
    private val iconsCache = LruCache<String, Drawable>(500)

    @WorkerThread
    @Throws(PackageManager.NameNotFoundException::class)
    fun getAppIcon(packageName: String): Drawable {
        check(!Looper.getMainLooper().isCurrentThread) { "Should not be called from MainThread" }
        iconsCache.get(packageName)?.let {
            return it
        }
        return packageManager.getApplicationIcon(packageName).also {
            iconsCache.put(packageName, it)
        }
    }

    fun dispose() {
        iconsCache.evictAll()
    }
}
