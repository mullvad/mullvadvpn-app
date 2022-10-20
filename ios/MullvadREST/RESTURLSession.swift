//
//  RESTURLSession.swift
//  MullvadREST
//
//  Created by pronebird on 18/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension REST {
    public static func makeURLSession() -> URLSession {
        let certificatePath = Bundle(for: SSLPinningURLSessionDelegate.self)
            .path(forResource: "le_root_cert", ofType: "cer")!
        let data = FileManager.default.contents(atPath: certificatePath)!
        let secCertificate = SecCertificateCreateWithData(nil, data as CFData)!

        let sessionDelegate = SSLPinningURLSessionDelegate(
            sslHostname: defaultAPIHostname,
            trustedRootCertificates: [secCertificate]
        )

        let session = URLSession(
            configuration: .ephemeral,
            delegate: sessionDelegate,
            delegateQueue: nil
        )

        return session
    }
}
