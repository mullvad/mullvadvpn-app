//
//  NSRegularExpression+IPAddress.swift
//  MullvadVPN
//
//  Created by pronebird on 30/10/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
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

        return try! NSRegularExpression(pattern: pattern, options: [.allowCommentsAndWhitespace])
    }

    static var ipv6RegularExpression: NSRegularExpression {
        // Regular expression obtained from:
        // https://stackoverflow.com/a/17871737
        let pattern = #"""
            # IPv6 RegEx
            (
            ([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|          # 1:2:3:4:5:6:7:8
            ([0-9a-fA-F]{1,4}:){1,7}:|                         # 1::                              1:2:3:4:5:6:7::
            ([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|         # 1::8             1:2:3:4:5:6::8  1:2:3:4:5:6::8
            ([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|  # 1::7:8           1:2:3:4:5::7:8  1:2:3:4:5::8
            ([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|  # 1::6:7:8         1:2:3:4::6:7:8  1:2:3:4::8
            ([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|  # 1::5:6:7:8       1:2:3::5:6:7:8  1:2:3::8
            ([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|  # 1::4:5:6:7:8     1:2::4:5:6:7:8  1:2::8
            [0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|       # 1::3:4:5:6:7:8   1::3:4:5:6:7:8  1::8
            :((:[0-9a-fA-F]{1,4}){1,7}|:)|                     # ::2:3:4:5:6:7:8  ::2:3:4:5:6:7:8 ::8       ::
            fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|     # fe80::7:8%eth0   fe80::7:8%1     (link-local IPv6 addresses with zone index)
            ::(ffff(:0{1,4}){0,1}:){0,1}
            ((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}
            (25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|          # ::255.255.255.255   ::ffff:255.255.255.255  ::ffff:0:255.255.255.255  (IPv4-mapped IPv6 addresses and IPv4-translated addresses)
            ([0-9a-fA-F]{1,4}:){1,4}:
            ((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}
            (25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])           # 2001:db8:3:4::192.0.2.33  64:ff9b::192.0.2.33 (IPv4-Embedded IPv6 Address)
            )
        """#

        return try! NSRegularExpression(pattern: pattern, options: [.allowCommentsAndWhitespace])
    }
}
