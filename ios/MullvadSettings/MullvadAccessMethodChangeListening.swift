// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

// A protocol that listens for notifications of when the current access method has changed. It receives only the UUID of the new method.
public protocol MullvadAccessMethodChangeListening: Sendable, AnyObject {
    func accessMethodChangedTo(_ uuid: UUID)
}
