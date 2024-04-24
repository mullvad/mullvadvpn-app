//
//  RelaySelection.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-04-29.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

struct RelaySelection {
    enum MultihopContext: Int, CaseIterable, CustomStringConvertible {
        case entry, exit

        var description: String {
            switch self {
            case .entry:
                NSLocalizedString(
                    "MULTIHOP_ENTRY",
                    tableName: "SelectLocation",
                    value: "Entry",
                    comment: ""
                )
            case .exit:
                NSLocalizedString(
                    "MULTIHOP_EXIT",
                    tableName: "SelectLocation",
                    value: "Exit",
                    comment: ""
                )
            }
        }
    }

    var entry: UserSelectedRelays?
    var exit: UserSelectedRelays?
    var currentContext: MultihopContext

    var invertedContext: MultihopContext {
        switch currentContext {
        case .entry:
            .exit
        case .exit:
            .entry
        }
    }

    var relaysFromCurrentContext: (current: UserSelectedRelays?, inverted: UserSelectedRelays?) {
        switch currentContext {
        case .entry:
            (current: entry, inverted: exit)
        case .exit:
            (current: exit, inverted: entry)
        }
    }
}
