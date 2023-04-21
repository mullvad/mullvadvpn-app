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
        public var localPort: UInt16

        public init(address: String = "127.0.0.1", port: UInt16) {
            self.address = address
            self.localPort = port
        }

        fileprivate func apply(to sessionConfiguration: URLSessionConfiguration) {
            var configuration = [CFString: Any]()

//            let forcedHost = "192.168.1.166"
//            let forcedPort = 2055
//
//            configuration[("SOCKSEnable" as NSString)] = 1
//            configuration[("SOCKSProxy" as NSString)] = forcedHost
//            configuration[("SOCKSPort" as NSString)] = NSNumber(value: forcedPort)
//            configuration[kCFProxyTypeKey] = kCFProxyTypeSOCKS
//            configuration[kCFStreamPropertySOCKSVersion] = kCFStreamSocketSOCKSVersion5
            
            configuration[kCFNetworkProxiesHTTPEnable] = NSNumber(value: 1)
            configuration[kCFNetworkProxiesHTTPProxy] = address as CFString
            configuration[kCFNetworkProxiesHTTPPort] = NSNumber(value: localPort)
            configuration[kCFNetworkProxiesProxyAutoConfigEnable] = kCFBooleanFalse

            configuration["HTTPSEnable" as CFString] = NSNumber(value: 1)
            configuration["HTTPSProxy" as CFString] = address as CFString
            configuration["HTTPSPort" as CFString] = NSNumber(value: localPort)
            

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
        sessionConfiguration.urlCache = nil
        httpProxyConfiguration?.apply(to: sessionConfiguration)
        
        let session = URLSession(
            configuration: sessionConfiguration,
            delegate: sessionDelegate,
            delegateQueue: nil
        )
        
        return session
    }
}
