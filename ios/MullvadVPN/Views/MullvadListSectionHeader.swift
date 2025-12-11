import SwiftUI

struct MullvadListSectionHeader: View {
    let title: LocalizedStringKey
    let subtitle: LocalizedStringKey?
    
    init(title: LocalizedStringKey, subtitle: LocalizedStringKey? = nil) {
        self.title = title
        self.subtitle = subtitle
    }

    var body: some View {
        HStack {
            Text(title)
                .font(.mullvadTiny)
                .foregroundStyle(Color.mullvadTextPrimary)
                .layoutPriority(1)
            Rectangle()
                .frame(height: 1)
                .foregroundStyle(Color.mullvadTextPrimary.opacity(0.2))
            if let subtitle {
                Text(subtitle)
                    .font(.mullvadTiny)
                    .foregroundStyle(Color.mullvadTextPrimary)
                    .layoutPriority(1)
            }
        }
        .frame(minHeight: 44, alignment: .center)
    }
}

#Preview {
    MullvadListSectionHeader(title: "Custom lists")
}
