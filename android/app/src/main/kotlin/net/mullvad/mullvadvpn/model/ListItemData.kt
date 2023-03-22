package net.mullvad.mullvadvpn.model

import androidx.annotation.DrawableRes
import androidx.annotation.IntDef
import androidx.annotation.StringRes

data class ListItemData
private constructor(
    val identifier: String,
    val text: String? = null,
    @StringRes val textRes: Int? = null,
    @DrawableRes val iconRes: Int?,
    val isSelected: Boolean,
    @ItemType val type: Int,
    val widget: WidgetState? = null,
    val action: ItemAction? = null
) {

    @Retention @IntDef(DIVIDER, PLAIN, ACTION) annotation class ItemType

    class Builder(private val identifier: String) {
        var text: String? = null

        @StringRes var textRes: Int? = null

        @DrawableRes var iconRes: Int? = null
        var isSelected: Boolean = false

        @ItemType var type: Int = 0
        var widget: WidgetState? = null
        var action: ItemAction? = null

        fun build(): ListItemData {
            if ((this.text == null && this.textRes == null) && type > PROGRESS)
                throw IllegalArgumentException("ListItem should be configured with text")

            return ListItemData(
                this.identifier,
                this.text,
                this.textRes,
                this.iconRes,
                this.isSelected,
                this.type,
                this.widget,
                this.action
            )
        }
    }

    data class ItemAction(val identifier: String)

    companion object {
        const val DIVIDER = 0
        const val PROGRESS = 1
        const val PLAIN = 2
        const val ACTION = 3
        const val DOUBLE_ACTION = 4
        const val APPLICATION = 5
        fun build(identifier: String, setUp: Builder.() -> Unit): ListItemData =
            Builder(identifier).also(setUp).build()
    }
}
