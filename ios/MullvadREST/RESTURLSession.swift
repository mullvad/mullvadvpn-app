//
//  RESTURLSession.swift
//  MullvadREST
//
//  Created by pronebird on 18/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension REST {
    public struct HTTPProxyConfiguration {
        public var address: String
        public var port: UInt16

        public init(address: String, port: UInt16) {
            self.address = address
            self.port = port
        }

        fileprivate func apply(to sessionConfiguration: URLSessionConfiguration) {
            var configuration = [CFString: Any]()

            configuration[kCFNetworkProxiesHTTPProxy] = address
            configuration[kCFNetworkProxiesHTTPPort] = NSNumber(value: port)
            configuration[kCFNetworkProxiesProxyAutoConfigEnable] = kCFBooleanFalse

            sessionConfiguration.connectionProxyDictionary = configuration
        }
    }

    public static func makeURLSession(httpProxyConfiguration: HTTPProxyConfiguration? = nil) -> URLSession {
        let certificatePath = Bundle(for: SSLPinningURLSessionDelegate.self)
            .path(forResource: "le_root_cert", ofType: "cer")!
        let data = FileManager.default.contents(atPath: certificatePath)!
        let secCertificate = SecCertificateCreateWithData(nil, data as CFData)!

        let sessionDelegate = SSLPinningURLSessionDelegate(
            sslHostname: defaultAPIHostname,
            trustedRootCertificates: [secCertificate]
        )

        let sessionConfiguration = URLSessionConfiguration.ephemeral
        httpProxyConfiguration?.apply(to: sessionConfiguration)

        let session = URLSession(
            configuration: sessionConfiguration,
            delegate: sessionDelegate,
            delegateQueue: nil
        )
        
        return session
    }
}
