//
//  CustomListSaveAlert.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-08.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

class CustomListSaveAlertViewModel: ObservableObject {
    @Published var inputIsValid: Bool

    init(inputIsValid: Bool = true) {
        self.inputIsValid = inputIsValid
    }
}

struct CustomListSaveAlert: View {
    @State private var inputText = ""
    @ObservedObject var viewModel: CustomListSaveAlertViewModel

    var didTapSave: ((String) -> Void)?
    var didTapCancel: (() -> Void)?

    var body: some View {
        GeometryReader(content: { geometry in
            ZStack {
                VStack(alignment: .leading, spacing: 8) {
                    Text("Edit list name")
                        .font(.system(size: 12, weight: .semibold))
                        .foregroundColor(.white.opacity(0.8))
                    TextField("", text: $inputText)
                        .textFieldStyle(CustomTextFieldStyle())
                    ErrorLabel(inputIsValid: viewModel.inputIsValid)
                    VStack(spacing: 16) {
                        SUIAppButton(text: "Save", style: .success)
                            .frame(height: 42)
                            .onTapGesture { didTapSave?(inputText) }
                        SUIAppButton(text: "Cancel", style: .default)
                            .frame(height: 42)
                            .onTapGesture { didTapCancel?() }
                    }
                }
                .frame(
                    minWidth: 0,
                    maxWidth: geometry.size.width > UIMetrics.preferredFormSheetContentSize.width
                        ? UIMetrics.preferredFormSheetContentSize.width
                        : .infinity
                )
                .padding(EdgeInsets(UIMetrics.CustomAlert.containerMargins))
                .background(Color(.secondaryColor))
                .cornerRadius(11)
            }
            .padding(EdgeInsets(UIMetrics.CustomAlert.containerMargins))
            .frame(minWidth: 0, maxWidth: .infinity, minHeight: 0, maxHeight: .infinity)
            .background(Color.black.opacity(0.6))
            .ignoresSafeArea()
        })
    }
}

struct CustomListSaveAlert_Previews: PreviewProvider {
    static var previews: some View {
        CustomListSaveAlert(viewModel: CustomListSaveAlertViewModel(inputIsValid: true))
        CustomListSaveAlert(viewModel: CustomListSaveAlertViewModel(inputIsValid: false))
    }
}

private struct CustomTextFieldStyle: TextFieldStyle {
    func _body(configuration: TextField<Self._Label>) -> some View {
        configuration
            .padding(EdgeInsets(top: 6, leading: 8, bottom: 6, trailing: 8))
            .background(Color.white)
            .cornerRadius(4)
            .font(.system(size: 15))
    }
}

private struct ErrorLabel: View {
    let inputIsValid: Bool

    var body: some View {
        Text("Name is already taken")
            .font(.system(size: 12, weight: .semibold))
            .foregroundColor(Color(.dangerColor))
            .padding(.bottom, inputIsValid ? 0 : 4)
            .opacity(inputIsValid ? 0 : 1)
    }
}
