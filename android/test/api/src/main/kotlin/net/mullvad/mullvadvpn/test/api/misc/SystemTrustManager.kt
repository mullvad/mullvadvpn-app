package net.mullvad.mullvadvpn.test.api.misc

import android.annotation.SuppressLint
import java.security.KeyStore
import java.security.cert.CertPathValidator
import java.security.cert.CertificateException
import java.security.cert.CertificateFactory
import java.security.cert.PKIXParameters
import java.security.cert.TrustAnchor
import java.security.cert.X509Certificate
import javax.net.ssl.X509TrustManager

// The app under test declares a domain-specific `network-security-config` (to permit cleartext
// traffic to localhost for the gRPC/UDS connection to the daemon). Since instrumented tests run
// in the app's process, that config applies to them too. Android's default trust manager
// (obtained via the "PKIX"/"X509" algorithm) is then backed by
// `android.security.net.config.RootTrustManager`, which throws a `CertificateException` for any
// TLS connection performed via the hostname-unaware `checkServerTrusted(chain, authType)`
// overload, since it can't tell which per-domain config to use without a hostname. Ktor's CIO
// engine only ever calls that hostname-unaware overload.
//
// Android intentionally does not register a fallback `TrustManagerFactory` implementation for
// "PKIX"/"X509" (the `RootTrustManager` above is the only one available), so there is no way to
// get a plain trust manager via `TrustManagerFactory`. Instead, we perform PKIX certificate chain
// validation ourselves, against the system's trusted CA certificates, using `CertPathValidator`.
// That API is not wrapped by the network-security-config machinery (only `TrustManagerFactory`
// is), so it works regardless of any domain-config entries. Ktor's CIO engine performs its own
// separate hostname verification, so this does not weaken certificate validation.
fun systemTrustManager(): X509TrustManager = SystemTrustManager

@SuppressLint("CustomX509TrustManager")
private object SystemTrustManager : X509TrustManager {
    private val validator = CertPathValidator.getInstance("PKIX")
    private val certificateFactory = CertificateFactory.getInstance("X.509")

    private val trustAnchors: Set<TrustAnchor> by lazy {
        val keyStore = KeyStore.getInstance("AndroidCAStore").apply { load(null, null) }
        keyStore
            .aliases()
            .asSequence()
            .mapNotNull { alias -> keyStore.getCertificate(alias) as? X509Certificate }
            .map { certificate -> TrustAnchor(certificate, null) }
            .toSet()
    }

    override fun checkClientTrusted(chain: Array<out X509Certificate>, authType: String) {
        throw CertificateException("Client certificates are not supported")
    }

    override fun checkServerTrusted(chain: Array<out X509Certificate>, authType: String) {
        val certPath = certificateFactory.generateCertPath(chain.toList())
        val params = PKIXParameters(trustAnchors).apply { isRevocationEnabled = false }
        validator.validate(certPath, params)
    }

    override fun getAcceptedIssuers(): Array<X509Certificate> =
        trustAnchors.map { it.trustedCert }.toTypedArray()
}
