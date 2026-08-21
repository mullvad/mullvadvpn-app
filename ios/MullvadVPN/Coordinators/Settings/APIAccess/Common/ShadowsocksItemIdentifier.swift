// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation

/// Item identifier used by diffable data sources implementing shadowsocks configuration.
enum ShadowsocksItemIdentifier: Hashable, CaseIterable {
    case server
    case port
    case password
    case cipher

    /// Cell identifier for the item identifier.
    var cellIdentifier: AccessMethodCellReuseIdentifier {
        switch self {
        case .server, .port, .password:
            .textInput
        case .cipher:
            .textWithDisclosure
        }
    }

    /// Indicates whether cell representing the item should be selectable.
    var isSelectable: Bool {
        self == .cipher
    }

    /// The text describing the item identifier and suitable to be used as a field label.
    var text: String {
        switch self {
        case .server:
            NSLocalizedString("Server", comment: "")
        case .port:
            NSLocalizedString("Port", comment: "")
        case .password:
            NSLocalizedString("Password", comment: "")
        case .cipher:
            NSLocalizedString("Cipher", comment: "")
        }
    }
}
