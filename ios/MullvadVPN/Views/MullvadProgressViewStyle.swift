import SwiftUI

struct MullvadProgressViewStyle: ProgressViewStyle {
    let size: CGFloat

    init(size: CGFloat = 48) {
        self.size = size
    }

    @State var isAnimating = false
    func makeBody(configuration: Configuration) -> some View {
        Image.mullvadIconSpinner
            .resizable()
            .frame(maxWidth: size, maxHeight: size)
            .rotationEffect(.degrees(isAnimating ? 360 : 0))
            .onAppear {
                withAnimation(
                    .linear(duration: 0.6).repeatForever(autoreverses: false)
                ) {
                    isAnimating = true
                }
            }
    }
}

#Preview {
    ProgressView()
        .progressViewStyle(MullvadProgressViewStyle())
        .background(Color.mullvadBackground)
}
