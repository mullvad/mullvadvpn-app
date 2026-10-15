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
import MullvadTypes
import SwiftUI

struct AddEditApiAccessMethodView: View {
    @Binding var name: String
    let onSave: (PersistentAccessMethod) -> Void

    private let itemFactory = SegmentedListItemFactory()

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                SegmentedListItem(
                    isLastInList: true,
                    userInteraction: .enabledWithoutHighlight,
                    accessibilityIdentifier: .accessMethodEnableSwitch,
                    leading: {
                        itemFactory.leading(for: .generic(title: NSLocalizedString("Name", comment: "")))
                    },
                    trailing: {
                        ConfigurableTextField(
                            placeholder: LocalizedStringKey("Required"), text: $name,
                            borderStyle: .constant(.none),
                            appearance: InputViewAppearance(backgroundColor: .clear))

                    }
                )
                .padding(.bottom, 24.0)

                SegmentedListItem(
                    isLastInList: true,
                    userInteraction: .enabledWithoutHighlight,
                    accessibilityIdentifier: .accessMethodEnableSwitch,
                    leading: {
                        itemFactory.leading(for: .generic(title: NSLocalizedString("Name", comment: "")))
                    },
                    trailing: {
                        itemFactory.trailing(for: .input(title: "", placeholder: "Required", text: $name))

                    }
                )
                .padding(.bottom, 24.0)
            }
        }
        .scrollBounceBehavior(.automatic)
        .padding(.horizontal, 16)
        .padding(.bottom, 16)
        .background(Color.mullvadBackground)
    }
}

#Preview {
    AddEditApiAccessMethodView(name: .constant("")) { accessMethod in
        print(accessMethod.name)
    }
}
