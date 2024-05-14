package net.mullvad.mullvadvpn.lib.daemon.grpc.util

import android.util.Log
import io.grpc.CallOptions
import io.grpc.Channel
import io.grpc.ClientCall
import io.grpc.ClientInterceptor
import io.grpc.MethodDescriptor

internal class LogInterceptor(private val logTag: String) : ClientInterceptor {
    override fun <ReqT : Any?, RespT : Any?> interceptCall(
        method: MethodDescriptor<ReqT, RespT>?,
        callOptions: CallOptions?,
        next: Channel?
    ): ClientCall<ReqT, RespT> {
        Log.d(logTag, "Intercepted call: ${method?.fullMethodName}")
        return next!!.newCall(method, callOptions)
    }
}
