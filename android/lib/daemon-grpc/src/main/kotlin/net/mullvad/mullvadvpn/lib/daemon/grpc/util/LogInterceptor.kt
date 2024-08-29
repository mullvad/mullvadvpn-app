package net.mullvad.mullvadvpn.lib.daemon.grpc.util

import co.touchlab.kermit.Logger
import io.grpc.CallOptions
import io.grpc.Channel
import io.grpc.ClientCall
import io.grpc.ClientInterceptor
import io.grpc.MethodDescriptor

internal class LogInterceptor : ClientInterceptor {
    override fun <ReqT : Any?, RespT : Any?> interceptCall(
        method: MethodDescriptor<ReqT, RespT>?,
        callOptions: CallOptions?,
        next: Channel?,
    ): ClientCall<ReqT, RespT> {
        Logger.v("Intercepted call: ${method?.fullMethodName}")
        return next!!.newCall(method, callOptions)
    }
}
