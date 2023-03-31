//
//  ConnectedWifiNetwork.swift
//  MullvadVPN
//
//  Created by pronebird on 31/03/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import NetworkExtension
import SystemConfiguration.CaptiveNetwork

/**
 Struct that represents information about Wi-Fi network device is connected to.
 */
struct ConnectedWifiNetwork: Equatable {
    /// Hotspot SSID.
    let ssid: String

    /// Hotspot BSSID.
    let bssid: String

    /// Hotspot security type.
    let securityType: String?

    static func fetchCurrent(_ completion: @escaping (ConnectedWifiNetwork?) -> Void) {
        if #available(iOS 14.0, *) {
            NEHotspotNetwork.fetchCurrent { network in
                completion(network.map { hotspotNetwork in
                    var securityType: String?

                    if #available(iOS 15.0, *) {
                        securityType = hotspotNetwork.securityType.description
                    }

                    return ConnectedWifiNetwork(
                        ssid: hotspotNetwork.ssid,
                        bssid: hotspotNetwork.bssid,
                        securityType: securityType
                    )
                })
            }
        } else {
            let interfaces = CNCopySupportedInterfaces() as? [CFString] ?? []

            for interface in interfaces {
                if let networkInfo = CNCopyCurrentNetworkInfo(interface) as NSDictionary?,
                   let ssid = networkInfo.object(forKey: kCNNetworkInfoKeySSID) as? String,
                   let bssid = networkInfo.object(forKey: kCNNetworkInfoKeyBSSID) as? String
                {
                    completion(ConnectedWifiNetwork(ssid: ssid, bssid: bssid, securityType: nil))
                    return
                }
            }

            completion(nil)
        }
    }
}

@available(iOS 15.0, *)
extension NEHotspotNetworkSecurityType {
    var description: String {
        switch self {
        case .WEP:
            return "WEP"
        case .enterprise:
            return "Enterprise"
        case .open:
            return "Open"
        case .personal:
            return "Personal"
        case .unknown:
            fallthrough
        @unknown default:
            return "Unknown"
        }
    }
}
