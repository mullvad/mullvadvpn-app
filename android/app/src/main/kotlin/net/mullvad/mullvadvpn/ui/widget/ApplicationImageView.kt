package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.content.pm.PackageManager
import android.graphics.drawable.Drawable
import android.util.AttributeSet
import androidx.appcompat.widget.AppCompatImageView
import androidx.core.content.res.ResourcesCompat
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancelChildren
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.applist.ApplicationsIconManager
import net.mullvad.mullvadvpn.di.APPS_SCOPE
import org.koin.core.scope.KoinScopeComponent
import org.koin.core.scope.Scope
import org.koin.core.scope.inject

class ApplicationImageView
@JvmOverloads
constructor(
    context: Context,
    attrs: AttributeSet? = null,
    defStyleAttr: Int = R.attr.applicationListItemViewStyle,
) : AppCompatImageView(context, attrs, defStyleAttr), KoinScopeComponent {
    private val viewScope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)
    private val iconManager: ApplicationsIconManager by inject()

    private var updateImageJob: Job? = null

    override val scope: Scope = getKoin().getScope(APPS_SCOPE)

    var packageName: String = ""
        set(value) {
            field = value
            updateImage()
        }

    init {
        updateImage(ResourcesCompat.getDrawable(resources, R.drawable.ic_icons_missing, null)!!)
    }

    private fun updateImage() {
        updateImageJob?.cancel()
        updateImageJob = viewScope.launch { loadImage()?.let { drawable -> updateImage(drawable) } }
    }

    override fun onAttachedToWindow() {
        super.onAttachedToWindow()
        updateImage()
    }

    override fun onDetachedFromWindow() {
        super.onDetachedFromWindow()
        updateImageJob?.cancel()
        viewScope.coroutineContext.cancelChildren()
    }

    private suspend fun loadImage(): Drawable? =
        withContext(Dispatchers.Default) {
            try {
                iconManager.getAppIcon(packageName)
            } catch (e: PackageManager.NameNotFoundException) {
                null
            }
        }

    private fun updateImage(drawable: Drawable) = setImageDrawable(drawable)
}
