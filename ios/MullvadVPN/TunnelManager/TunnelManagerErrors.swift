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
import MullvadTypes

struct UnsetTunnelError: LocalizedError {
    var errorDescription: String? {
        NSLocalizedString("Tunnel is unset.", comment: "")
    }
}

struct InvalidDeviceStateError: LocalizedError {
    var errorDescription: String? {
        NSLocalizedString("Invalid device state.", comment: "")
    }
}

struct StartTunnelError: LocalizedError, WrappingError {
    private let _underlyingError: Error

    var errorDescription: String? {
        NSLocalizedString("Failed to start the tunnel.", comment: "")
    }

    var underlyingError: Error? {
        _underlyingError
    }

    init(underlyingError: Error) {
        _underlyingError = underlyingError
    }
}

struct StopTunnelError: LocalizedError, WrappingError {
    private let _underlyingError: Error

    var errorDescription: String? {
        NSLocalizedString("Failed to stop the tunnel.", comment: "")
    }

    var underlyingError: Error? {
        _underlyingError
    }

    init(underlyingError: Error) {
        _underlyingError = underlyingError
    }
}
