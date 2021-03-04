package net.mullvad.mullvadvpn.ui.widget

import android.animation.ValueAnimator
import android.app.Activity
import android.content.Context
import android.util.AttributeSet
import android.view.LayoutInflater
import android.view.View
import android.view.View.MeasureSpec
import android.view.View.OnLayoutChangeListener
import android.view.inputmethod.InputMethodManager
import android.widget.RelativeLayout
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.ListItemDividerDecoration
import net.mullvad.mullvadvpn.ui.LoginState
import net.mullvad.mullvadvpn.ui.widget.AccountLoginBorder.BorderState
import net.mullvad.mullvadvpn.util.Debouncer

class AccountLogin : RelativeLayout {
    companion object {
        private val MAX_ACCOUNT_HISTORY_ENTRIES = 3
    }

    private val focusDebouncer = Debouncer(false).apply {
        listener = { hasFocus -> focused = hasFocus }
    }

    private val container =
        context.getSystemService(Context.LAYOUT_INFLATER_SERVICE).let { service ->
            val inflater = service as LayoutInflater

            inflater.inflate(R.layout.account_login, this)
        }

    private val border: AccountLoginBorder = container.findViewById(R.id.border)
    private val accountHistoryList: RecyclerView = container.findViewById(R.id.history)
    private val input: AccountInput = container.findViewById(R.id.input)

    private val historyAdapter = AccountHistoryAdapter().apply {
        onSelectEntry = { account -> input.loginWith(account) }
        onChildFocusChanged = { _, hasFocus -> focusDebouncer.rawValue = hasFocus }
    }

    private val dividerHeight = resources.getDimensionPixelSize(R.dimen.account_history_divider)
    private val historyEntryHeight =
        resources.getDimensionPixelSize(R.dimen.account_history_entry_height)

    private val historyAnimation = ValueAnimator.ofInt(0, 0).apply {
        addUpdateListener { animation ->
            updateHeight(animation.animatedValue as Int)
        }

        duration = 350
    }

    private val maxHeight: Int
        get() = MAX_ACCOUNT_HISTORY_ENTRIES * (historyEntryHeight + dividerHeight)

    private val expandedHeight: Int
        get() = collapsedHeight + historyHeight

    private var historyHeight by observable(0) { _, oldHistoryHeight, newHistoryHeight ->
        if (newHistoryHeight != oldHistoryHeight) {
            historyAnimation.setIntValues(collapsedHeight, expandedHeight)
            reposition()
        }
    }

    private var collapsedHeight by observable(
        calculateInitialInputHeight()
    ) { _, oldCollapsedHeight, newCollapsedHeight ->
        if (newCollapsedHeight != oldCollapsedHeight) {
            historyAnimation.setIntValues(newCollapsedHeight, expandedHeight)
            reposition()
        }
    }

    private var focused by observable(false) { _, _, hasFocus ->
        updateBorder()
        shouldShowAccountHistory = hasFocus

        if (!hasFocus) {
            hideKeyboard()
        }
    }

    private var shouldShowAccountHistory by observable(false) { _, isShown, show ->
        if (isShown != show) {
            if (show) {
                historyAnimation.start()
            } else {
                historyAnimation.reverse()
            }
        }
    }

    val hasFocus
        get() = focused

    var accountHistory by observable<ArrayList<String>?>(null) { _, _, history ->
        val entryCount = history?.size ?: 0

        historyHeight = entryCount * (historyEntryHeight + dividerHeight)

        if (history != null) {
            historyAdapter.accountHistory = history
        }
    }

    var state: LoginState by observable(LoginState.Initial) { _, _, newState ->
        input.loginState = newState

        updateBorder()

        if (newState == LoginState.Success) {
            visibility = View.INVISIBLE
        }
    }

    var onLogin: ((String) -> Unit)?
        get() = input.onLogin
        set(value) { input.onLogin = value }

    var onRemoveFromHistory: ((String) -> Unit)?
        get() = historyAdapter.onRemoveEntry
        set(value) { historyAdapter.onRemoveEntry = value }

    constructor(context: Context) : super(context) {}

    constructor(context: Context, attributes: AttributeSet) : super(context, attributes) {}

    constructor(context: Context, attributes: AttributeSet, defaultStyleAttribute: Int) :
        super(context, attributes, defaultStyleAttribute) {}

    constructor(
        context: Context,
        attributes: AttributeSet,
        defaultStyleAttribute: Int,
        defaultStyleResource: Int
    ) : super(context, attributes, defaultStyleAttribute, defaultStyleResource) {
    }

    init {
        border.elevation = elevation + 0.1f

        input.apply {
            onFocusChanged.subscribe(this) { hasFocus ->
                focusDebouncer.rawValue = hasFocus
            }

            onTextChanged.subscribe(this) { _ ->
                if (state == LoginState.Failure) {
                    state = LoginState.Initial
                }
            }

            addOnLayoutChangeListener(
                OnLayoutChangeListener { _, _, top, _, bottom, _, _, _, _ ->
                    collapsedHeight = bottom - top
                }
            )
        }

        accountHistoryList.apply {
            layoutManager = LinearLayoutManager(context)
            adapter = historyAdapter

            addItemDecoration(
                ListItemDividerDecoration(context).apply {
                    topOffsetId = R.dimen.account_history_divider
                }
            )
        }
    }

    fun onDestroy() {
        input.onFocusChanged.unsubscribe(this)
        input.onTextChanged.unsubscribe(this)
    }

    private fun calculateInitialInputHeight(): Int {
        if (input.height == 0) {
            val widthMeasureSpec = MeasureSpec.makeMeasureSpec(0, MeasureSpec.EXACTLY)
            val heightMeasureSpec = MeasureSpec.makeMeasureSpec(0, MeasureSpec.UNSPECIFIED)

            input.measure(widthMeasureSpec, heightMeasureSpec)
        }

        return input.height
    }

    private fun updateBorder() {
        if (state == LoginState.Failure) {
            border.borderState = BorderState.ERROR
        } else if (focused) {
            border.borderState = BorderState.FOCUSED
        } else {
            border.borderState = BorderState.UNFOCUSED
        }
    }

    private fun updateHeight(height: Int) {
        val layoutParams = container.layoutParams as MarginLayoutParams

        layoutParams.height = height
        layoutParams.bottomMargin = maxHeight - height

        container.layoutParams = layoutParams
    }

    private fun reposition() {
        historyAnimation.cancel()

        if (shouldShowAccountHistory) {
            updateHeight(expandedHeight)
        } else {
            updateHeight(collapsedHeight)
        }
    }

    private fun hideKeyboard() {
        val inputManagerId = Activity.INPUT_METHOD_SERVICE
        val inputManager = context.getSystemService(inputManagerId) as InputMethodManager

        inputManager.hideSoftInputFromWindow(windowToken, 0)
    }
}
