//
//  MultihopActionDescriptor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-05-18.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import Foundation
import MullvadSettings
import SwiftUI

struct MultihopActionDescriptor: SettingsMigrationPresentable {
    let action: SuggestedAction<MultihopSuggestedAction>

    var title: String {
        switch action.kind {
        case .multihopWhenNeeded:
            return NSLocalizedString(
                "Suggested action",
                comment: ""
            )

        case .automaticEntry:
            return NSLocalizedString(
                "Suggested multihop entry",
                comment: ""
            )
        }
    }

    var banner: Image? {
        return nil
    }

    var description: [MullvadStateView.TextItem] {
        switch action.kind {
        case .multihopWhenNeeded:
            return [
                MullvadStateView.TextItem(
                    text: String(
                        format: NSLocalizedString(
                            """
                            To avoid getting blocked, we recommend that you set your multihop mode to “%@”.
                            """,
                            comment: ""
                        ), MultihopStateV2.whenNeeded.description),
                    style: .secondary()
                ),

                MullvadStateView.TextItem(
                    text: NSLocalizedString(
                        """
                        This mode allows the app to automatically multihop through an additional server if needed to ensure your current settings work with your selected location.
                        """,
                        comment: ""
                    ),
                    style: .secondary()
                ),

                MullvadStateView.TextItem(
                    text: NSLocalizedString(
                        """
                        Attention: In this mode, filters are ignored for the additional server.
                        """,
                        comment: ""
                    ),
                    style: .secondary(.bold)
                ),
            ]

        case .automaticEntry:
            return [
                MullvadStateView.TextItem(
                    text: String(
                        format: NSLocalizedString(
                            """
                            To avoid having to change the entry server manually, we recommend you set the multihop entry server to “%@”.
                            """,
                            comment: ""
                        ), NSLocalizedString("Automatic", comment: "")),
                    style: .secondary()
                ),

                MullvadStateView.TextItem(
                    text: NSLocalizedString(
                        """
                        When selected, the app automatically picks a random server, prioritizing those closer to the exit location for better performance.
                        """,
                        comment: ""
                    ),
                    style: .secondary()
                ),

                MullvadStateView.TextItem(
                    text: String(
                        format: NSLocalizedString(
                            """
                            Attention: With the “%@” location, filters are ignored for the entry server.
                            """,
                            comment: ""
                        ), NSLocalizedString("Automatic", comment: "")),
                    style: .secondary(.bold)
                ),
            ]
        }
    }

    func makeState(for kind: MullvadStateView.ActionState.Kind) -> MullvadStateView.ActionState {
        switch kind {
        case .idle:
            idle
        case .loading:
            loading
        case .success:
            success
        case .failure:
            failure
        }
    }

    private var idle: MullvadStateView.ActionState {
        switch action.kind {
        case .multihopWhenNeeded:
            MullvadStateView.ActionState(
                kind: .idle,
                message: String(
                    format: NSLocalizedString(
                        "Change to “%@”",
                        comment: ""
                    ),
                    arguments: [MultihopStateV2.whenNeeded.description]
                ))

        case .automaticEntry:
            MullvadStateView.ActionState(
                kind: .idle,
                message: String(
                    format: NSLocalizedString(
                        "Set entry to “%@”",
                        comment: ""
                    ),
                    arguments: [
                        NSLocalizedString(
                            "Automatic",
                            comment: ""
                        )
                    ]
                ))
        }
    }

    private var loading: MullvadStateView.ActionState {
        switch action.kind {
        case .multihopWhenNeeded:
            MullvadStateView.ActionState(
                kind: .loading,
                message: NSLocalizedString(
                    "Changing mode...",
                    comment: ""
                ))

        case .automaticEntry:
            MullvadStateView.ActionState(
                kind: .loading,
                message: NSLocalizedString(
                    "Setting entry...",
                    comment: ""
                ))
        }
    }

    private var success: MullvadStateView.ActionState {
        switch action.kind {
        case .multihopWhenNeeded:
            MullvadStateView.ActionState(
                kind: .success,
                message: NSLocalizedString(
                    "Multihop mode changed",
                    comment: ""
                ))

        case .automaticEntry:
            MullvadStateView.ActionState(
                kind: .success,
                message: String(
                    format: NSLocalizedString(
                        "Entry set to “%@”",
                        comment: ""), NSLocalizedString("Automatic", comment: "")))
        }
    }

    private var failure: MullvadStateView.ActionState {
        switch action.kind {
        case .multihopWhenNeeded:
            MullvadStateView.ActionState(
                kind: .success,
                message: NSLocalizedString(
                    "Failed to change mode",
                    comment: ""
                ))

        case .automaticEntry:
            MullvadStateView.ActionState(
                kind: .success,
                message: String(
                    format: NSLocalizedString(
                        "Failed to set the Entry to “%@”",
                        comment: ""), NSLocalizedString("Automatic", comment: "")))
        }
    }
}
