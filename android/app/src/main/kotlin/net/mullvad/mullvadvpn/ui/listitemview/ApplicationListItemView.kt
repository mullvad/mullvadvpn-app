package net.mullvad.mullvadvpn.ui.listitemview

import android.content.Context
import android.content.pm.PackageManager
import android.graphics.drawable.Drawable
import android.util.AttributeSet
import androidx.core.content.res.ResourcesCompat
import androidx.core.view.isVisible
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
import org.koin.core.component.KoinApiExtension
import org.koin.core.scope.KoinScopeComponent
import org.koin.core.scope.Scope
import org.koin.core.scope.inject

@OptIn(KoinApiExtension::class)
class ApplicationListItemView @JvmOverloads constructor(
    context: Context,
    attrs: AttributeSet? = null,
    defStyleAttr: Int = R.attr.applicationListItemViewStyle,
    defStyleRes: Int = 0
) : ActionListItemView(context, attrs, defStyleAttr, defStyleRes), KoinScopeComponent {
    private val viewScope = CoroutineScope(SupervisorJob() + Dispatchers.Main.immediate)
    private val iconManager: ApplicationsIconManager by inject()

    private var updateImageJob: Job? = null

    override val scope: Scope = getKoin().getScope(APPS_SCOPE)

    init {
        itemText.setTextAppearance(R.style.TextAppearance_Mullvad_Title2)
        updateImage(ResourcesCompat.getDrawable(resources, R.drawable.ic_icons_missing, null)!!)
    }

    override fun updateImage() {
        itemIcon.isVisible = true
        updateImageJob?.cancel()
        updateImageJob = viewScope.launch {
            loadImage()?.let { drawable ->
                updateImage(drawable)
            }
        }
    }

    override fun updateText() {
        itemData.text?.let {
            itemText.text = it
        }
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

    private suspend fun loadImage(): Drawable? = withContext(Dispatchers.Default) {
        try {
            iconManager.getAppIcon(itemData.identifier)
        } catch (e: PackageManager.NameNotFoundException) {
            null
        }
    }

    private fun updateImage(drawable: Drawable) = itemIcon.setImageDrawable(drawable)
}
