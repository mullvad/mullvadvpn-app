package net.mullvad.mullvadvpn.ui.listitemview

import android.content.Context
import android.content.pm.PackageManager
import android.content.res.Resources
import android.util.AttributeSet
import android.util.Log
import androidx.core.content.res.ResourcesCompat
import androidx.core.view.isVisible
import kotlinx.android.synthetic.main.list_item_base.view.*
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.ListItemData
import org.koin.core.component.KoinApiExtension
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject

@OptIn(KoinApiExtension::class)
class ApplicationListItemView @JvmOverloads constructor(
    context: Context,
    attrs: AttributeSet? = null,
    defStyleAttr: Int = R.attr.applicationListItemViewStyle,
    defStyleRes: Int = 0
) : ActionListItemView(context, attrs, defStyleAttr, defStyleRes), KoinComponent {
    private val packageManager: PackageManager by inject()

    private var appResources: Resources? = null

    init {
        itemText.setTextAppearance(R.style.TextAppearance_Mullvad_Title2)
    }

    override fun update(data: ListItemData) {
        data.action?.identifier?.let {
            appResources = packageManager.getResourcesForApplication(it)
        }
        super.update(data)
    }

    override fun updateImage() {
        itemIcon.isVisible = true
        itemData.iconRes?.let { iconRes ->
            appResources?.let { appRes ->
                itemIcon.setImageDrawable(ResourcesCompat.getDrawable(appRes, iconRes, null))
            }
        }
    }

    override fun updateText() {
        itemData.text?.let {
            itemText.text = it
        }
    }
}
