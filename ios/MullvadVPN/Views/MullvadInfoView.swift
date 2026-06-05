import SwiftUI

struct MullvadInfoView: View {
    let bodyText: String
    let link: String

    let onTapLink: (() -> Void)?

    var body: some View {
        var headerText: AttributedString {
            var bodyText = AttributedString(bodyText)
            bodyText.foregroundColor = Color(.ContentHeading.textColor)
            var link = AttributedString(link)
            link.foregroundColor = Color(.ContentHeading.linkColor)
            return bodyText + link
        }
        Button {
            onTapLink?()
        } label: {
            Text(headerText)
                .font(.mullvadTiny)
                .multilineTextAlignment(.leading)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
    }
}
