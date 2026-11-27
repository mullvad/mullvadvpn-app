package net.mullvad.mullvadvpn.lib.billing

import android.content.Context
import com.android.billingclient.api.BillingClient
import com.android.billingclient.api.BillingClientStateListener
import com.android.billingclient.api.BillingResult
import com.android.billingclient.api.PendingPurchasesParams
import com.android.billingclient.api.PurchasesUpdatedListener
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import net.mullvad.mullvadvpn.lib.billing.model.BillingException

class BillingClientConnection(
    private val context: Context,
    private val purchaseUpdateListener: PurchasesUpdatedListener,
    // We use a single thread so that we are not doing multiple parallel connections
    private val connectionStateDispatcher: CoroutineDispatcher =
        Dispatchers.Default.limitedParallelism(1),
) {
    private var client: BillingClient? = null

    private val connectionState = MutableStateFlow<ConnectionState>(ConnectionState.Disconnected)

    @Suppress("ThrowsCount")
    suspend fun getClient(): BillingClient {
        return withTimeoutOrNull(10.seconds) {
            withContext(connectionStateDispatcher) {
                val client = client ?: initClient()
                when (client.connectionState) {
                    BillingClient.ConnectionState.DISCONNECTED -> {
                        connectionState.value = ConnectionState.Connecting
                        startConnection(client)
                    }
                    BillingClient.ConnectionState.CLOSED -> {
                        val newClient = initClient()
                        connectionState.value = ConnectionState.Connecting
                        startConnection(newClient)
                    }
                    BillingClient.ConnectionState.CONNECTING -> {
                        // Nothing to do just wait for the other connection to finish
                    }
                    BillingClient.ConnectionState.CONNECTED -> {
                        // We are already connected
                        // Update the connectionState flow if it has a stale state
                        connectionState.value = ConnectionState.Connected
                    }
                }
            }

            // Wait for the state to be not connecting
            val state = connectionState.first { it !is ConnectionState.Connecting }

            when (state) {
                ConnectionState.Closed ->
                    throw BillingException(
                        responseCode = BillingClient.BillingResponseCode.SERVICE_DISCONNECTED,
                        message = "Billing client is closed",
                    )
                ConnectionState.Connected ->
                    client ?: error("Illegal state exception billing client is null")
                ConnectionState.Disconnected ->
                    throw BillingException(
                        responseCode = BillingClient.BillingResponseCode.SERVICE_DISCONNECTED,
                        message = "Billing client is disconnected",
                    )
                is ConnectionState.Error -> throw state.exception
                else -> error("Unexpected state: $state")
            }
        }
            ?: throw BillingException(
                responseCode = BillingClient.BillingResponseCode.SERVICE_DISCONNECTED,
                message = "Timeout waiting for billing client to connect",
            )
    }

    fun endConnection() {
        client?.endConnection()
        connectionState.value = ConnectionState.Closed
        client = null
    }

    private fun startConnection(client: BillingClient) {
        client.startConnection(
            object : BillingClientStateListener {
                override fun onBillingSetupFinished(billingResult: BillingResult) {
                    if (client == this@BillingClientConnection.client) {
                        if (billingResult.responseCode == BillingClient.BillingResponseCode.OK) {
                            connectionState.value = ConnectionState.Connected
                        } else {
                            connectionState.value =
                                ConnectionState.Error(
                                    BillingException(
                                        responseCode = billingResult.responseCode,
                                        message = billingResult.debugMessage,
                                    ),
                                )
                        }
                    }
                }

                override fun onBillingServiceDisconnected() {
                    if (client == this@BillingClientConnection.client) {
                        connectionState.value = ConnectionState.Disconnected
                    }
                }
            },
        )
    }

    private fun initClient() =
        BillingClient.newBuilder(context)
            .setListener(purchaseUpdateListener)
            .enablePendingPurchases(
                PendingPurchasesParams.newBuilder().enableOneTimeProducts().build(),
            )
            .build()
            .also { client = it }
}

private sealed interface ConnectionState {
    data object Disconnected : ConnectionState

    data object Connecting : ConnectionState

    data object Connected : ConnectionState

    data class Error(val exception: BillingException) : ConnectionState

    data object Closed : ConnectionState
}
