// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Combine
import MullvadTypes

public enum RecentConnectionsResult {
    case success(RecentConnections)
    case failure(Error)
}
public protocol RecentConnectionsRepositoryProtocol {
    var recentConnectionsPublisher: AnyPublisher<RecentConnectionsResult, Never> { get }
    func disable()
    func enable(
        _ selectedEntryConstraint: RelayConstraint<UserSelectedRelays>?,
        selectedExitConstraint: RelayConstraint<UserSelectedRelays>)
    func add(
        _ selectedEntryConstraint: RelayConstraint<UserSelectedRelays>,
        selectedExitConstraint: RelayConstraint<UserSelectedRelays>)
    func deleteCustomList(_ id: UUID)
    func load()
}
