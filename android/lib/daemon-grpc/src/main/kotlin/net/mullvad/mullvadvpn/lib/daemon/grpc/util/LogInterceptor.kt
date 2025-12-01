package net.mullvad.mullvadvpn.lib.daemon.grpc.util

import co.touchlab.kermit.Logger
import okhttp3.Interceptor
import okhttp3.Response

internal class LogInterceptor : Interceptor {
    /*override fun <ReqT : Any?, RespT : Any?> interceptCall(
        method: MethodDescriptor<ReqT, RespT>?,
        callOptions: CallOptions?,
        next: Channel?,
    ): ClientCall<ReqT, RespT> {
        Logger.v("Intercepted call: ${method?.fullMethodName}")
        return next!!.newCall(method, callOptions)
    }*/

    override fun intercept(chain: Interceptor.Chain): Response {
        Logger.v("Intercepted call: ${chain.request()}")
        return chain.proceed(chain.request()).also { Logger.v("Intercepted response $it") }
    }
}
