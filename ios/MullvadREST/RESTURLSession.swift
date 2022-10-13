//
//  RESTURLSession.swift
//  MullvadVPN
//
//  Created by pronebird on 18/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension REST {
    static let sharedURLSession: URLSession = {
        let certificatePath = Bundle.main.path(forResource: "le_root_cert", ofType: "cer")!
        let data = FileManager.default.contents(atPath: certificatePath)!
        let secCertificate = SecCertificateCreateWithData(nil, data as CFData)!

        let sessionDelegate = SSLPinningURLSessionDelegate(
            sslHostname: ApplicationConfiguration.defaultAPIHostname,
            trustedRootCertificates: [secCertificate]
        )

        let session = URLSession(
            configuration: .ephemeral,
            delegate: sessionDelegate,
            delegateQueue: nil
        )

        return session
    }()
}
