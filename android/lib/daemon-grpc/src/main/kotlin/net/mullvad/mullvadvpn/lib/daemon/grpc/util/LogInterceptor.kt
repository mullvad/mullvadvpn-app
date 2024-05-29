package net.mullvad.mullvadvpn.lib.daemon.grpc.util

import android.util.Log
import io.grpc.CallOptions
import io.grpc.Channel
import io.grpc.ClientCall
import io.grpc.ClientInterceptor
import io.grpc.MethodDescriptor
import net.mullvad.mullvadvpn.lib.common.constant.TAG

internal class LogInterceptor : ClientInterceptor {
    override fun <ReqT : Any?, RespT : Any?> interceptCall(
        method: MethodDescriptor<ReqT, RespT>?,
        callOptions: CallOptions?,
        next: Channel?
    ): ClientCall<ReqT, RespT> {
        Log.d(TAG, "Intercepted call: ${method?.fullMethodName}")
        return next!!.newCall(method, callOptions)
    }
}
