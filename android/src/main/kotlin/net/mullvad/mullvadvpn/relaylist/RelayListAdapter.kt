package net.mullvad.mullvadvpn.relaylist

import java.lang.ref.WeakReference
import java.util.LinkedList

import android.support.v7.widget.RecyclerView.Adapter
import android.view.LayoutInflater
import android.view.ViewGroup

import net.mullvad.mullvadvpn.R

class RelayListAdapter(private val initialSelectedItemCode: String?) : Adapter<RelayItemHolder>() {
    private val relayList = fakeRelayList
    private val activeIndices = LinkedList<WeakReference<RelayListAdapterPosition>>()
    private var selectedItem: RelayItem? = null
    private var selectedItemHolder: RelayItemHolder? = null

    var onSelect: ((String?) -> Unit)? = null

    init {
        initialSelectedItemCode?.let { code -> selectedItem = findRelayItemByCode(code) }
    }

    override fun onCreateViewHolder(parentView: ViewGroup, type: Int): RelayItemHolder {
        val inflater = LayoutInflater.from(parentView.context)
        val view = inflater.inflate(R.layout.relay_list_item, parentView, false)
        val index = RelayListAdapterPosition(0)

        activeIndices.add(WeakReference(index))

        return RelayItemHolder(view, this, index)
    }

    override fun onBindViewHolder(holder: RelayItemHolder, position: Int) {
        var remaining = position

        for (country in relayList) {
            val itemOrCount = country.getItem(remaining)

            when (itemOrCount) {
                is GetItemResult.Item -> {
                    bindHolderToItem(holder, itemOrCount.item, position)
                    return
                }
                is GetItemResult.Count -> remaining -= itemOrCount.count
            }
        }
    }

    override fun getItemCount() = relayList.map { country -> country.visibleItemCount }.sum()

    fun selectItem(item: RelayItem?, holder: RelayItemHolder?) {
        selectedItemHolder?.selected = false

        selectedItem = item
        selectedItemHolder = holder
        selectedItemHolder?.apply { selected = true }

        onSelect?.invoke(item?.code)
    }

    fun expandItem(itemIndex: RelayListAdapterPosition, childCount: Int) {
        val position = itemIndex.position

        updateActiveIndices(position, childCount)
        notifyItemRangeInserted(position + 1, childCount)
    }

    fun collapseItem(itemIndex: RelayListAdapterPosition, childCount: Int) {
        val position = itemIndex.position

        updateActiveIndices(position, -childCount)
        notifyItemRangeRemoved(position + 1, childCount)
    }

    private fun updateActiveIndices(position: Int, delta: Int) {
        val activeIndicesIterator = activeIndices.iterator()

        while (activeIndicesIterator.hasNext()) {
            val index = activeIndicesIterator.next().get()

            if (index == null) {
                activeIndicesIterator.remove()
            } else {
                val indexPosition = index.position

                if (indexPosition > position) {
                    index.position = indexPosition + delta
                }
            }
        }
    }

    private fun bindHolderToItem(holder: RelayItemHolder, item: RelayItem, position: Int) {
        holder.item = item
        holder.itemPosition.position = position

        if (selectedItem != null && selectedItem == item) {
            holder.selected = true
            selectedItemHolder = holder
        } else {
            holder.selected = false
        }
    }

    private fun findRelayItemByCode(code: String): RelayItem? {
        val codeParts = code.split('-')

        for (country in relayList) {
            if (country.code == codeParts[0]) {
                if (codeParts.size == 1) {
                    return country
                } else {
                    var relayCode: String? = null

                    if (codeParts.size == 3) {
                        relayCode = codeParts[2]
                    }

                    return country.findRelayItemByCode("${codeParts[0]}-${codeParts[1]}", relayCode)
                }
            }
        }

        return null
    }
}

val fakeRelayList = listOf(
    RelayCountry(
        "Australia",
        "au",
        false,
        listOf(
            RelayCity(
                "Brisbane",
                "au-bne",
                false,
                listOf(Relay("au-bne-001"))
            ),
            RelayCity(
                "Melbourne",
                "au-mel",
                false,
                listOf(Relay("au-mel-002"), Relay("au-mel-003"), Relay("au-mel-004"))
            ),
            RelayCity(
                "Perth",
                "au-per",
                false,
                listOf(Relay("au-per-001"))
            ),
            RelayCity(
                "Sydney",
                "au-syd",
                false,
                listOf(
                    Relay("au1-wireguard"),
                    Relay("au-syd-001"),
                    Relay("au-syd-002"),
                    Relay("au-mel-003")
                )
            )
        )
    ),
    RelayCountry(
        "South Africa",
        "za",
        false,
        listOf(
            RelayCity(
                "Johannesburg",
                "za-jnb",
                false,
                listOf(Relay("za-jnb-001"))
            )
        )
    ),
    RelayCountry(
        "Sweden",
        "se",
        false,
        listOf(
            RelayCity(
                "Gothenburg",
                "se-got",
                false,
                listOf(
                    Relay("se3-wireguard"),
                    Relay("se5-wireguard"),
                    Relay("se-got-001"),
                    Relay("se-got-002"),
                    Relay("se-got-003"),
                    Relay("se-got-004"),
                    Relay("se-got-005"),
                    Relay("se-got-006"),
                    Relay("se-got-007")
                )
            ),
            RelayCity(
                "Helsingborg",
                "se-hel",
                false,
                listOf(
                    Relay("se-hel-001"),
                    Relay("se-hel-002"),
                    Relay("se-hel-003"),
                    Relay("se-hel-004"),
                    Relay("se-hel-007"),
                    Relay("se-hel-008")
                )
            ),
            RelayCity(
                "Malmö",
                "se-mma",
                false,
                listOf(
                    Relay("se4-wireguard"),
                    Relay("se-mma-001"),
                    Relay("se-mma-002"),
                    Relay("se-mma-003"),
                    Relay("se-mma-004"),
                    Relay("se-mma-005"),
                    Relay("se-mma-006"),
                    Relay("se-mma-007"),
                    Relay("se-mma-008"),
                    Relay("se-mma-009"),
                    Relay("se-mma-010")
                )
            ),
            RelayCity(
                "Stockholm",
                "se-sto",
                false,
                listOf(
                    Relay("se2-wireguard"),
                    Relay("se6-wireguard"),
                    Relay("se7-wireguard"),
                    Relay("se8-wireguard"),
                    Relay("se-sto-001"),
                    Relay("se-sto-002"),
                    Relay("se-sto-003"),
                    Relay("se-sto-004"),
                    Relay("se-sto-005"),
                    Relay("se-sto-006"),
                    Relay("se-sto-007"),
                    Relay("se-sto-008"),
                    Relay("se-sto-009"),
                    Relay("se-sto-010"),
                    Relay("se-sto-011"),
                    Relay("se-sto-012"),
                    Relay("se-sto-013"),
                    Relay("se-sto-014"),
                    Relay("se-sto-015"),
                    Relay("se-sto-016"),
                    Relay("se-sto-017"),
                    Relay("se-sto-018"),
                    Relay("se-sto-019"),
                    Relay("se-sto-020"),
                    Relay("se-sto-021"),
                    Relay("se-sto-022"),
                    Relay("se-sto-023"),
                    Relay("se-sto-024"),
                    Relay("se-sto-025")
                )
            )
        )
    )
)
