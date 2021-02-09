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
        bind()
    }

    abstract fun updateState(state: T)
    abstract fun bind()

    class StandardController(parent: ViewGroup) : WidgetViewController<WidgetState.ImageState>(parent) {
        override val layoutRes: Int
            get() = R.layout.list_item_widget_image
        private lateinit var imageView: ImageView
        override fun bind() {
            imageView = parent.findViewById(R.id.widgetImage)
        }

        override fun updateState(state: WidgetState.ImageState) =
            imageView.setImageResource(state.imageRes)
    }
    class SwitchController(parent: ViewGroup) : WidgetViewController<WidgetState.SwitchState>(parent) {
        override val layoutRes: Int
            get() = R.layout.list_item_widget_switch
        private lateinit var switch: SwitchCompat

        override fun bind() {
            switch = parent.findViewById(R.id.widgetSwitch)
        }
        override fun updateState(state: WidgetState.SwitchState) {
            switch.isChecked = state.isChecked
        }
    }
}
