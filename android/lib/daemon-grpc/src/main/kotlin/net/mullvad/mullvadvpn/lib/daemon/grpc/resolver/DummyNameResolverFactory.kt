package net.mullvad.mullvadvpn.lib.daemon.grpc.resolver

import io.grpc.NameResolver
import java.net.URI

class DummyNameResolverFactory : NameResolver.Factory() {
    override fun newNameResolver(targetUri: URI, args: NameResolver.Args): NameResolver {
        return DummyNameResolver()
    }

    override fun getDefaultScheme(): String {
        return DNS_SCHEME
    }

    companion object {
        private const val DNS_SCHEME = "dns"
    }
}
