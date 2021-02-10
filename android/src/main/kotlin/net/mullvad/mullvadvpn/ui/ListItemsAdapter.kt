package net.mullvad.mullvadvpn.ui

import android.view.ViewGroup
import androidx.recyclerview.widget.AsyncDifferConfig
import androidx.recyclerview.widget.AsyncListDiffer
import androidx.recyclerview.widget.DiffUtil
import androidx.recyclerview.widget.ListUpdateCallback
import androidx.recyclerview.widget.RecyclerView
import net.mullvad.mullvadvpn.model.ListItemData
import net.mullvad.mullvadvpn.ui.listitemview.ActionListItemView
import net.mullvad.mullvadvpn.ui.listitemview.ApplicationListItemView
import net.mullvad.mullvadvpn.ui.listitemview.DividerGroupListItemView
import net.mullvad.mullvadvpn.ui.listitemview.ListItemView
import net.mullvad.mullvadvpn.ui.listitemview.PlainListItemView
import net.mullvad.mullvadvpn.ui.listitemview.ProgressListItemView

class ListItemsAdapter : RecyclerView.Adapter<ListItemsAdapter.ViewHolder>() {

    var listItemListener: ListItemListener? = null

    protected var updateCallback: ListUpdateCallback? = null

    protected var diffCallback: DiffCallback = DefaultDiffCallback()

    private val listDiffer: AsyncListDiffer<ListItemData> = createDiffer(diffCallback)

    fun setItems(items: List<ListItemData?>) = listDiffer.submitList(items)

    override fun onCreateViewHolder(parent: ViewGroup, @ListItemData.ItemType viewType: Int):
        ListItemsAdapter.ViewHolder {
            return ViewHolder(
                when (viewType) {
                    ListItemData.DIVIDER -> DividerGroupListItemView(parent.context)
                    ListItemData.PROGRESS -> ProgressListItemView(parent.context)
                    ListItemData.PLAIN -> PlainListItemView(parent.context)
                    ListItemData.ACTION -> ActionListItemView(parent.context)
                    ListItemData.APPLICATION -> ApplicationListItemView(parent.context)
                    else ->
                        throw IllegalArgumentException("View type /'$viewType/' is not supported")
                }
            )
        }

    override fun onBindViewHolder(holder: ViewHolder, position: Int) {
        (holder.itemView as ListItemView).update(getItem(position))
        holder.itemView.listItemListener = listItemListener
    }

    override fun getItemCount(): Int = listDiffer.currentList.size

    @ListItemData.ItemType
    override fun getItemViewType(position: Int): Int = getItem(position).type

    private fun getItem(position: Int): ListItemData = listDiffer.currentList[position]

    private fun createDiffer(diffCallback: DiffCallback): AsyncListDiffer<ListItemData> {
        return AsyncListDiffer(getListUpdateCallback(), getConfig(diffCallback))
    }

    private fun getConfig(diffUtil: DiffCallback): AsyncDifferConfig<ListItemData> {
        return AsyncDifferConfig.Builder(diffUtil).build()
    }

    protected fun getListUpdateCallback(): ListUpdateCallback {
        return object : ListUpdateCallback {
            override fun onInserted(position: Int, count: Int) {
                updateCallback?.onInserted(position, count)
                notifyItemRangeInserted(position, count)
            }

            override fun onRemoved(position: Int, count: Int) {
                updateCallback?.onRemoved(position, count)
                notifyItemRangeRemoved(position, count)
            }

            override fun onMoved(fromPosition: Int, toPosition: Int) {
                updateCallback?.onMoved(fromPosition, toPosition)
                notifyItemMoved(fromPosition, toPosition)
            }

            override fun onChanged(position: Int, count: Int, payload: Any?) {
                updateCallback?.onChanged(position, count, payload)
                notifyItemRangeChanged(position, count, payload)
            }
        }
    }

    internal class DefaultDiffCallback : DiffCallback() {
        override fun areItemsTheSame(oldItem: ListItemData, newItem: ListItemData): Boolean {
            return oldItem.type == newItem.type && oldItem.identifier == newItem.identifier
        }

        override fun areContentsTheSame(oldItem: ListItemData, newItem: ListItemData): Boolean {
            return oldItem == newItem
        }
    }

    inner class ViewHolder(view: ListItemView) : RecyclerView.ViewHolder(view)
}
typealias DiffCallback = DiffUtil.ItemCallback<ListItemData>
