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

struct ConfigureApiAccessMethodView: View {
    @State var viewModel: any ConfigureApiAccessMethodViewModelProtocol
    @Environment(\.dismiss) private var dismiss
    @State private var savingAlert: MullvadAlert?
    @FocusState private var focusedInput: AccessMethodField?
    @State private var lastFocusedInput: AccessMethodField?
    @State private var hasAppeared: Bool = false
    let testViewAnchor = "testView"

    private let itemFactory = SegmentedListItemFactory()

    init(viewModel: any ConfigureApiAccessMethodViewModelProtocol) {
        _viewModel = State(initialValue: viewModel)
    }

    var body: some View {
        ScrollViewReader { proxy in
            ScrollView {
                VStack(alignment: .leading, spacing: 0) {
                    promptView
                    nameField
                    typeField
                    serverDetailsSection
                    testView
                }
                .padding(.horizontal, 16)
            }
            .scrollBounceBehavior(.basedOnSize)
            .scrollDismissesKeyboard(.interactively)
            .onChange(of: viewModel.testState) {
                if viewModel.testState == .testing {
                    withAnimation {
                        proxy.scrollTo(
                            testViewAnchor,
                            anchor: .center
                        )
                    }
                } else if case .reachable = viewModel.testState {
                    viewModel.save()
                    dismiss()
                } else if case .unreachable = viewModel.testState {
                    savingAlert = methodSavingAlert
                }
            }
        }
        .background {
            Color.mullvadBackground
                .ignoresSafeArea()
        }
        .accessibilityIdentifier(.addAccessMethodScrollView)
        .navigationBarTitleDisplayMode(.inline)
        .mullvadAlert(item: $savingAlert)
        .onAppear {
            guard !hasAppeared else { return }
            hasAppeared = true

            focusedInput = .name

            LocalNetworkProbe()
                .triggerLocalNetworkPrivacyAlert()
        }
        .onDisappear {
            focusedInput = nil
            viewModel.stopTesting()
        }
        .onChange(of: focusedInput) { _, newValue in
            if let newValue {
                lastFocusedInput = newValue
            }
        }
        .onChange(
            of: viewModel.accessMethodViewModel.method,
            {
                focusedInput = lastFocusedInput
            }
        )
        .doneBarButton(
            accessibilityIdentifier: .accessMethodAddButton,
            isDisabled: !viewModel.canSubmit
        ) {
            focusedInput = nil
            viewModel.startTesting()
        }
    }

    private var promptView: some View {
        HStack(alignment: .center) {
            Spacer()
            ResizableImageView(image: .mullvadIconInfo, dimension: .width(15))
            Text("The app will test the method before saving.")
                .font(.mullvadTinySemiBold)
                .frame(maxWidth: .infinity, alignment: .leading)
                .foregroundStyle(Color.mullvadTextSecondary)
            Spacer()
        }
        .padding(.bottom, 16)
    }

    private var nameField: some View {
        Group {
            SegmentedListItem(
                isLastInList: true,
                userInteraction: .enabledWithoutHighlight,
                accessibilityIdentifier: .accessMethodNameTextField,
                borderStyle: .constant(viewModel.borderStyle(for: .name)),
                leading: {
                    AccessMethodLabelView(title: "Name", textColor: viewModel.borderStyle(for: .name).textColor)
                },
                trailing: {
                    AccessMethodTextField(
                        placeholder: "Required", text: $viewModel.accessMethodViewModel.name, field: .name,
                        focusedField: $focusedInput)
                }
            )
            ValidationErrorView(error: viewModel.hasError(for: .name)?.errorDescription)
        }
        .onChange(of: viewModel.accessMethodViewModel.name) {
            viewModel.clearErrors(for: [.name])
        }
    }

    private var typeField: some View {
        SegmentedListItem(
            isLastInList: true,
            userInteraction: .enabledWithoutHighlight,
            accessibilityIdentifier: .accessMethodProtocolSelectionCell,
            leadingAndTrailingDestination: {
                AccessMethodTypeView(
                    viewModel: AccessMethodTypeViewModel(
                        current: Binding(
                            get: { viewModel.accessMethodViewModel.method.localizedDescription },
                            set: { newValue in
                                if let kind = AccessMethodKind.allUserDefinedKinds.first(where: {
                                    $0.localizedDescription == newValue
                                }) {
                                    viewModel.accessMethodViewModel.method = kind
                                }
                            }
                        ),
                        list: AccessMethodKind.allUserDefinedKinds.map(\.localizedDescription)
                    )
                )
            },
            leading: {
                itemFactory.leading(for: .generic(title: NSLocalizedString("Type", comment: "")))
            },
            trailing: {
                itemFactory.trailing(
                    for: .drillDown(title: viewModel.accessMethodViewModel.method.localizedDescription, breadcrumb: nil)
                )

            }
        )
        .padding(.vertical, 24.0)
        .simultaneousGesture(
            TapGesture().onEnded {
                focusedInput = nil
            }
        )
    }

    private var serverDetailsSection: some View {
        Group {
            if viewModel.accessMethodViewModel.method.hasProxyConfiguration {
                ServerDetailsSection(viewModel: viewModel, focusedInput: $focusedInput)
            } else {
                EmptyView()
            }
        }
    }

    private var testView: some View {
        MethodTestingStatusView(
            status: viewModel.testState,
            onStop: {
                viewModel.stopTesting()
            }
        )
        .id(testViewAnchor)
        .padding(.vertical, 24.0)
    }

    private var methodSavingAlert: MullvadAlert {
        MullvadAlert(
            type: .error,
            messages: ["API unreachable, save method anyway?"],
            actions: [
                .init(
                    type: .secondary,
                    title: "Save",
                    identifier: .accessMethodUnreachableSaveButton,
                    handler: {
                        savingAlert = nil
                        viewModel.save()
                        dismiss()
                    }),
                .init(
                    type: .primary,
                    title: "Cancel",
                    identifier: .accessMethodUnreachableCancelButton,
                    handler: {
                        savingAlert = nil
                    }
                ),
            ])
    }
}
