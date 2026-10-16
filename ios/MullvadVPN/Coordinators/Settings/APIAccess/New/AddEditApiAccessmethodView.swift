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

struct AddEditApiAccessMethodView<ViewModel: AddEditApiAccessMethodViewModelProtocol>: View {
    //@Binding var name: String
    let viewModel: ViewModel
    let onSave: (PersistentAccessMethod) -> Void
    @Environment(\.dismiss) private var dismiss
    private let itemFactory = SegmentedListItemFactory()

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 0) {
                    SegmentedListItem(
                        isLastInList: true,
                        userInteraction: .enabledWithoutHighlight,
                        leading: {
                            itemFactory.leading(for: .generic(title: NSLocalizedString("Name", comment: "")))
                        },
                        trailing: {
                            ConfigurableTextField(
                                placeholder: LocalizedStringKey("Required"), text: .constant(""),
                                borderStyle: .constant(.none),
                                appearance: InputViewAppearance(backgroundColor: .clear))

                        }
                    )
                    .accessibilityIdentifier(AccessibilityIdentifier.accessMethodNameTextField)
                    .padding(.bottom, 24.0)

                    SegmentedListItem(
                        isLastInList: true,
                        userInteraction: .enabledWithoutHighlight,
                        leadingAndTrailingDestination: {
                            Text("Next")
                                .navigationTitle("Next")
                                .navigationBarTitleDisplayMode(.inline)
                        },
                        leading: {
                            itemFactory.leading(for: .generic(title: NSLocalizedString("Type", comment: "")))
                        },
                        trailing: {
                            itemFactory.trailing(for: .drillDown(title: "", breadcrumb: nil))

                        }
                    )
                    .accessibilityIdentifier(AccessibilityIdentifier.accessMethodProtocolPickerView)
                    .padding(.bottom, 24.0)
                }
            }
            .scrollBounceBehavior(.automatic)
            .padding(.horizontal, 16)
            .padding(.bottom, 16)
            .background(Color.mullvadBackground)
            .safeAreaInset(edge: .top, spacing: 8.0) {
                VStack(spacing: 0) {
                    // Prompt
                    Text("The app will test the method before saving.")
                        .font(.mullvadTiny)
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 16)

                    // Navigation row
                    HStack {
                        Button(action: {
                            dismiss()
                        }) {
                            Image(systemName: "chevron.left")
                                .font(.system(size: 16, weight: .bold))
                                .foregroundStyle(.primary)
                                .frame(width: 32, height: 32)
                                .background(.white, in: Circle())
                        }

                        Text(viewModel.dismissButtonTitle)
                            .font(.mullvadSmall)

                        // title
                        Spacer()
                        Text(viewModel.title)
                            .font(.mullvadSmall)
                        Spacer()

                        Button("Save") {
                            // Your save action
                        }
                        .font(.mullvadSmall)
                    }
                    .padding(16.0)
                }
                .foregroundStyle(.white)
                .background(Color(red: 25 / 255, green: 48 / 255, blue: 69 / 255))
            }
            .toolbar(.hidden, for: .navigationBar)
        }
    }
}

#Preview {
    AddEditApiAccessMethodView(viewModel: AddEditApiAccessMethodViewModel(accessMethod: nil)) { accessMethod in
        print(accessMethod.name)
    }
}
