// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadSettings
import SwiftUI

struct QuantumResistanceView: View {
    private let itemFactory = SegmentedListItemFactory()
    @Binding var isQuantumResistanceEnabled: Bool
    @State private var alert: MullvadAlert?

    var body: some View {
        SegmentedListItem(
            userInteraction: .enabledWithoutHighlight,
            accessibilityIdentifier: .quantumResistantTunnelCell,
            leading: {
                // an orphan-control hack from https://stackoverflow.com/questions/70552173/is-there-a-way-to-disable-swiftuis-text-automatic-orphaning-fix
                // \u{200B} is a zero-width space, which prevents line breaking
                // there is a SwiftUI API for "properly" avoiding orphans, but it is not yet public, though should be moved to when it is: https://fatbobman.com/en/posts/controlling-orphans-in-swiftui-text/
                itemFactory.leading(
                    for: .generic(
                        title: NSLocalizedString("Quantum-resistant tunnel", comment: "")
                            + "\u{200B}\u{200B}\u{200B}\u{200B}\u{200B}\u{200B}")
                )
            },
            trailing: {
                itemFactory.trailing(
                    for: .custom(items: [
                        .button(
                            icon: .info,
                            onSelect: {
                                alert = getQuantumResistanceAlert(completion: { alert = nil })
                            },
                            sizing: .button
                        ),
                        .toggle(
                            isOn: $isQuantumResistanceEnabled,
                            isDisabled: false,
                            accessibilityID: isQuantumResistanceEnabled ? .quantumResistanceOff : .quantumResistanceOn
                        ),
                        .padding(),
                    ])
                )
            }
        )
        .mullvadAlert(item: $alert)
    }

    func getQuantumResistanceAlert(completion: @escaping () -> Void) -> MullvadAlert {
        MullvadAlert(
            type: .info,
            messages: [
                "This feature makes the WireGuard tunnel resistant to potential attacks from quantum computers.",
                """
                It does this by performing an extra key exchange using a quantum safe algorithm and mixing the result into WireGuard’s regular encryption. This extra step uses approximately 500 kiB of traffic every time a new tunnel is established.
                """,
            ], customView: nil,
            actions: [
                MullvadAlert.Action(type: .primary, title: "Got it!", handler: completion)
            ])
    }
}

#Preview {
    @Previewable @State var quantumResistance = false
    QuantumResistanceView(isQuantumResistanceEnabled: $quantumResistance)
}
