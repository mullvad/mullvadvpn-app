package net.mullvad.mullvadvpn.ui.listitemview

import android.view.LayoutInflater
import android.view.ViewGroup
import android.widget.ImageView
import androidx.annotation.LayoutRes
import androidx.appcompat.widget.SwitchCompat
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.WidgetState

sealed class WidgetViewController<T : WidgetState>(val parent: ViewGroup) {
    @get:LayoutRes
    protected abstract val layoutRes: Int

    init {
        LayoutInflater.from(parent.context).inflate(layoutRes, parent)
    }

    abstract fun updateState(state: T)

    class StandardController(parent: ViewGroup) :
        WidgetViewController<WidgetState.ImageState>(parent) {
        override val layoutRes: Int
            get() = R.layout.list_item_widget_image
        private val imageView: ImageView = parent.findViewById(R.id.widgetImage)
        override fun updateState(state: WidgetState.ImageState) =
            imageView.setImageResource(state.imageRes)
    }

    class SwitchController(parent: ViewGroup) :
        WidgetViewController<WidgetState.SwitchState>(parent) {
        override val layoutRes: Int
            get() = R.layout.list_item_widget_switch
        private val switch: SwitchCompat = parent.findViewById(R.id.widgetSwitch)
        override fun updateState(state: WidgetState.SwitchState) {
            switch.isChecked = state.isChecked
        }
    }
}
