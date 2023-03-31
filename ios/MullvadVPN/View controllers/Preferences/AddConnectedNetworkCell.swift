//
//  AddConnectedNetworkCell.swift
//  MullvadVPN
//
//  Created by pronebird on 31/03/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

class AddConnectedNetworkCell: SettingsAddEntryCell {
    var connectedNetwork: ConnectedWifiNetwork? {
        didSet {
            titleLabel.text = connectedNetwork?.cellTitle
            detailTitleLabel.text = connectedNetwork?.cellDescription
        }
    }

    override init(style: UITableViewCell.CellStyle, reuseIdentifier: String?) {
        super.init(style: .subtitle, reuseIdentifier: reuseIdentifier)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}

private extension ConnectedWifiNetwork {
    var cellTitle: String {
        return String(
            format: NSLocalizedString(
                "ADD_CONNECTED_NETWORK_BUTTON",
                tableName: "Preferences",
                value: "Add connected network: %@",
                comment: ""
            ), ssid
        )
    }

    var cellDescription: String {
        var description: [String] = []

        if let securityType = securityType {
            description.append(String(
                format: NSLocalizedString(
                    "ADD_CONNECTED_NETWORK_SECURITY_DESCRIPTION",
                    tableName: "Preferences",
                    value: "Security: %@",
                    comment: ""
                ),
                securityType
            ))
        }

        description.append(String(
            format: NSLocalizedString(
                "ADD_CONNECTED_NETWORK_BSSID_DESCRIPTION",
                tableName: "Preferences",
                value: "BSSID: %@",
                comment: ""
            ),
            bssid
        ))

        return description.joined(separator: ", ")
    }
}
