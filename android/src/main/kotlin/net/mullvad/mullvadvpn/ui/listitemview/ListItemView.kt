package net.mullvad.mullvadvpn.ui.listitemview

import android.annotation.SuppressLint
import android.content.Context
import android.util.AttributeSet
import android.view.LayoutInflater
import androidx.annotation.CallSuper
import androidx.annotation.DimenRes
import androidx.annotation.LayoutRes
import androidx.constraintlayout.widget.ConstraintLayout
import net.mullvad.mullvadvpn.model.ListItemData
import net.mullvad.mullvadvpn.ui.ListItemListener

abstract class ListItemView @JvmOverloads constructor(
    context: Context,
    attrs: AttributeSet? = null,
    defStyleAttr: Int = 0,
    defStyleRes: Int = 0
) : ConstraintLayout(context, attrs, defStyleAttr, defStyleRes) {

    private val inflater: LayoutInflater = LayoutInflater.from(context)

    protected lateinit var itemData: ListItemData

    var listItemListener: ListItemListener? = null

    @get:LayoutRes
    protected abstract val layoutRes: Int

    @get:DimenRes
    protected abstract val heightRes: Int

    init {
        init()
    }

    @SuppressLint("ResourceType")
    private fun init() {
        val view = inflater.inflate(layoutRes, this, true)
        val height = if (heightRes > 0)
            resources.getDimensionPixelSize(heightRes)
        else
            LayoutParams.WRAP_CONTENT
        view.layoutParams = LayoutParams(LayoutParams.MATCH_PARENT, height)
    }

    @CallSuper
    open fun update(data: ListItemData) {
        itemData = data
    }
}
