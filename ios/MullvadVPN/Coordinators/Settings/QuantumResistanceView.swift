//
//  QuantumResistanceView.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2026-07-30.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

struct QuantumResistanceView: View {
    private let itemFactory = SegmentedListItemFactory()
    @Binding var isQuantumResistanceEnabled: Bool
    @State private var alert: MullvadAlert?

    var body: some View {
        SegmentedListItem(
            userInteraction: .enabledWithoutHighlight,
            accessibilityIdentifier: .accessMethodAddButton,  // TODO: fix this
            leading: {
                itemFactory.leading(
                    for: .generic(title: NSLocalizedString("Quantum-resistant tunnel", comment: ""))
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
                            isDisabled: false
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
