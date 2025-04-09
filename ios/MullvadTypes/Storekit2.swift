public struct StorekitTransaction: Codable, Sendable {
    let transaction: String

    public init(transaction: String) {
        self.transaction = transaction
    }
}
