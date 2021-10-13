package net.mullvad.mullvadvpn.relaylist

import android.view.View
import android.view.ViewGroup.MarginLayoutParams
import android.widget.ImageButton
import android.widget.ImageView
import android.widget.TextView
import androidx.recyclerview.widget.RecyclerView.ViewHolder
import net.mullvad.mullvadvpn.R

class RelayItemHolder(
    private val view: View,
    private val adapter: RelayListAdapter,
    var itemPosition: RelayListAdapterPosition
) : ViewHolder(view) {
    private val name: TextView = view.findViewById(R.id.name)
    private val chevron: ImageButton = view.findViewById(R.id.chevron)
    private val clickArea: View = view.findViewById(R.id.click_area)
    private val relayStatus: View = view.findViewById(R.id.status)
    private val relayActive: ImageView = view.findViewById(R.id.relay_active)
    private val selectedIcon: View = view.findViewById(R.id.selected)

    private val context = view.context
    private val countryColor = context.getColor(R.color.blue)
    private val cityColor = context.getColor(R.color.blue40)
    private val relayColor = context.getColor(R.color.blue20)
    private val selectedColor = context.getColor(R.color.green)

    private val resources = view.resources
    private val countryPadding = resources.getDimensionPixelSize(R.dimen.country_row_padding)
    private val cityPadding = resources.getDimensionPixelSize(R.dimen.city_row_padding)
    private val relayPadding = resources.getDimensionPixelSize(R.dimen.relay_row_padding)

    var item: RelayItem? = null
        set(value) {
            field = value
            updateView()
        }

    var selected = false
        set(value) {
            field = value
            updateView()
        }

    init {
        chevron.setOnClickListener { toggle() }
        clickArea.setOnClickListener { adapter.selectItem(item, this) }
    }

    private fun updateView() {
        val item = this.item

        if (item != null) {
            name.text = item.name

            if (item.active) {
                name.alpha = 1.0F
            } else {
                name.alpha = 0.5F
            }

            if (selected) {
                relayActive.visibility = View.INVISIBLE
                selectedIcon.visibility = View.VISIBLE
            } else {
                relayActive.visibility = View.VISIBLE
                selectedIcon.visibility = View.INVISIBLE

                if (item.active) {
                    relayActive.setImageDrawable(adapter.activeRelayIcon)
                } else {
                    relayActive.setImageDrawable(adapter.inactiveRelayIcon)
                }
            }

            clickArea.setEnabled(item.active)

            when (item.type) {
                RelayItemType.Country -> setViewStyle(countryColor, countryPadding)
                RelayItemType.City -> setViewStyle(cityColor, cityPadding)
                RelayItemType.Relay -> setViewStyle(relayColor, relayPadding)
            }

            if (item.hasChildren) {
                chevron.visibility = View.VISIBLE

                if (item.expanded) {
                    chevron.rotation = 180.0F
                } else {
                    chevron.rotation = 0.0F
                }
            } else {
                chevron.visibility = View.GONE
            }
        } else {
            name.text = ""
            chevron.visibility = View.GONE
        }
    }

    private fun setViewStyle(rowColor: Int, padding: Int) {
        var backgroundColor = rowColor

        if (selected) {
            backgroundColor = selectedColor
        }

        (relayStatus.layoutParams as? MarginLayoutParams)?.let { parameters ->
            parameters.leftMargin = padding
            relayStatus.layoutParams = parameters
        }

        view.setBackgroundColor(backgroundColor)
    }

    private fun toggle() {
        item?.let { item ->
            if (!item.expanded) {
                item.expanded = true
                chevron.rotation = 180.0F
                adapter.expandItem(itemPosition, item.visibleChildCount)
            } else {
                val childCount = item.visibleChildCount

                item.expanded = false
                chevron.rotation = 0.0F
                adapter.collapseItem(itemPosition, childCount)
            }
        }
    }
}
