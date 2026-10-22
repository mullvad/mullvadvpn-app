// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

extension MullvadAlert {

    @MainActor
    static func deleteCustomListAlert(
        customList: LocationNode, onDelete: @escaping () -> Void, onCancel: @escaping () -> Void
    )
        -> MullvadAlert
    {
        .init(
            type: .warning,
            messages: ["Do you want to delete the list **\(customList.name)**?"],
            actions: [
                .init(
                    type: .destructivePrimary,
                    title: "Delete list",
                    identifier: .confirmDeleteCustomListButton,
                    handler: {
                        onDelete()
                    }
                ),
                .init(
                    type: .secondary,
                    title: "Cancel",
                    handler: {
                        onCancel()
                    }
                ),
            ]
        )
    }
}
