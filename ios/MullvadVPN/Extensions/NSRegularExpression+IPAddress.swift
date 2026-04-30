//
//  NSRegularExpression+IPAddress.swift
//  MullvadVPN
//
//  Created by pronebird on 30/10/2020.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension NSRegularExpression {
    static var ipv4RegularExpression: NSRegularExpression {
        // Regular expression obtained from:
        // https://www.regular-expressions.info/ip.html
        let pattern = #"""
            \b(25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.
              (25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.
              (25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\.
              (25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9])\b
            """#
        // swift-format-ignore: NeverUseForceTry
        return try! NSRegularExpression(pattern: pattern, options: [.allowCommentsAndWhitespace])
    }

    static var ipv6RegularExpression: NSRegularExpression {
        // Regular expression obtained from:
        // https://stackoverflow.com/a/17871737

        let ipv4Segment = "(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])"
        let ipv4Address = "(\(ipv4Segment)\\.){3,3}\(ipv4Segment)"

        let ipv6Segment = "[0-9a-fA-F]{1,4}"

        let long = "(\(ipv6Segment):){7,7}\(ipv6Segment)"
        let compressed1 = "(\(ipv6Segment):){1,7}:"
        let compressed2 = "(\(ipv6Segment):){1,6}:\(ipv6Segment)"
        let compressed3 = "(\(ipv6Segment):){1,5}(:\(ipv6Segment)){1,2}"
        let compressed4 = "(\(ipv6Segment):){1,4}(:\(ipv6Segment)){1,3}"
        let compressed5 = "(\(ipv6Segment):){1,3}(:\(ipv6Segment)){1,4}"
        let compressed6 = "(\(ipv6Segment):){1,2}(:\(ipv6Segment)){1,5}"
        let compressed7 = "\(ipv6Segment):((:\(ipv6Segment)){1,6})"
        let compressed8 = ":((:\(ipv6Segment)){1,7}|:)"

        let linkLocal = "[Ff][Ee]80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}"
        let ipv4Mapped = "::([fF]{4}(:0{1,4}){0,1}:){0,1}\(ipv4Address)"
        let ipv4Embedded = "(\(ipv6Segment):){1,4}:\(ipv4Address)"

        let pattern = [
            long,
            linkLocal,
            ipv4Mapped,
            ipv4Embedded,
            compressed8,
            compressed7,
            compressed6,
            compressed5,
            compressed4,
            compressed3,
            compressed2,
            compressed1,
        ].joined(separator: "|")
        // swift-format-ignore: NeverUseForceTry
        return try! NSRegularExpression(pattern: pattern, options: [.allowCommentsAndWhitespace])
    }
}
