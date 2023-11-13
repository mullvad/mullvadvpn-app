//
//  SSLPinningURLSessionDelegate.swift
//  MullvadREST
//
//  Created by pronebird on 17/05/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import Security

final class SSLPinningURLSessionDelegate: NSObject, URLSessionDelegate {
    private let sslHostnames: [String]
    private let trustedRootCertificates: [SecCertificate]

    private let logger = Logger(label: "SSLPinningURLSessionDelegate")

    init(sslHostnames: [String], trustedRootCertificates: [SecCertificate]) {
        self.sslHostnames = sslHostnames
        self.trustedRootCertificates = trustedRootCertificates
    }

    // MARK: - URLSessionDelegate

    func urlSession(
        _ session: URLSession,
        didReceive challenge: URLAuthenticationChallenge,
        completionHandler: @escaping (URLSession.AuthChallengeDisposition, URLCredential?) -> Void
    ) {
        if challenge.protectionSpace.authenticationMethod == NSURLAuthenticationMethodServerTrust,
           let serverTrust = challenge.protectionSpace.serverTrust,
           sslHostnames.first(where: { verifyServerTrust($0, serverTrust: serverTrust) }) != nil {
            completionHandler(.useCredential, URLCredential(trust: serverTrust))
        } else {
            completionHandler(.rejectProtectionSpace, nil)
        }
    }

    // MARK: - Private

    private func verifyServerTrust(_ sslHostname: String, serverTrust: SecTrust) -> Bool {
        var secResult: OSStatus

        // Set SSL policy
        let sslPolicy = SecPolicyCreateSSL(true, sslHostname as CFString)
        secResult = SecTrustSetPolicies(serverTrust, sslPolicy)
        guard secResult == errSecSuccess else {
            logger.error("SecTrustSetPolicies failure: \(formatErrorMessage(code: secResult))")
            return false
        }

        // Set trusted root certificates
        secResult = SecTrustSetAnchorCertificates(serverTrust, trustedRootCertificates as CFArray)
        guard secResult == errSecSuccess else {
            logger.error(
                "SecTrustSetAnchorCertificates failure: \(formatErrorMessage(code: secResult))"
            )
            return false
        }

        // Tell security framework to only trust the provided root certificates
        secResult = SecTrustSetAnchorCertificatesOnly(serverTrust, true)
        guard secResult == errSecSuccess else {
            logger.error(
                "SecTrustSetAnchorCertificatesOnly failure: \(formatErrorMessage(code: secResult))"
            )
            return false
        }

        var error: CFError?
        if SecTrustEvaluateWithError(serverTrust, &error) {
            return true
        } else {
            logger.error(
                "SecTrustEvaluateWithError failure: \(error?.localizedDescription ?? "<nil>")"
            )
            return false
        }
    }

    private func formatErrorMessage(code: OSStatus) -> String {
        let message = SecCopyErrorMessageString(code, nil) as String? ?? "<nil>"

        return "\(message) (code: \(code))"
    }
}
