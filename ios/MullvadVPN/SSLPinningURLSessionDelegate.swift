//
//  SSLPinningURLSessionDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 17/05/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging

class SSLPinningURLSessionDelegate: NSObject, URLSessionDelegate {

    private let trustedRootCertificates: [SecCertificate]
    private let logger = Logger(label: "SSLPinningURLSessionDelegate")

    init(trustedRootCertificates: [SecCertificate]) {
        self.trustedRootCertificates = trustedRootCertificates
    }

    // MARK: - URLSessionDelegate

    func urlSession(_ session: URLSession, didReceive challenge: URLAuthenticationChallenge, completionHandler: @escaping (URLSession.AuthChallengeDisposition, URLCredential?) -> Void) {
        let evaluation: (disposition: URLSession.AuthChallengeDisposition, credential: URLCredential?)

        if challenge.protectionSpace.authenticationMethod == NSURLAuthenticationMethodServerTrust {
            if let serverTrust = challenge.protectionSpace.serverTrust, self.verifyServerTrust(serverTrust) {
                evaluation = (.useCredential, URLCredential(trust: serverTrust))
            } else {
                evaluation = (.cancelAuthenticationChallenge, nil)
            }
        } else {
            evaluation = (.rejectProtectionSpace, nil)
        }

        completionHandler(evaluation.disposition, evaluation.credential)
    }


    // MARK: - Private

    private func verifyServerTrust(_ serverTrust: SecTrust) -> Bool {
        // Set trusted root certificates
        var secResult = SecTrustSetAnchorCertificates(serverTrust, trustedRootCertificates as CFArray)
        guard secResult == errSecSuccess else {
            self.logger.error("SecTrustSetAnchorCertificates failure: \(self.formatErrorMessage(code: secResult))")
            return false
        }

        // Tell security framework to only trust the provided root certificates
        secResult = SecTrustSetAnchorCertificatesOnly(serverTrust, true)
        guard secResult == errSecSuccess else {
            self.logger.error("SecTrustSetAnchorCertificatesOnly failure: \(self.formatErrorMessage(code: secResult))")
            return false
        }

        var error: CFError?
        if SecTrustEvaluateWithError(serverTrust, &error) {
            return true
        } else {
            self.logger.error("SecTrustEvaluateWithError failure: \(error?.localizedDescription ?? "<nil>")")
            return false
        }
    }

    private func formatErrorMessage(code: OSStatus) -> String {
        let message = SecCopyErrorMessageString(code, nil) as String? ?? "<nil>"

        return "\(message) (code: \(code))"
    }


}
