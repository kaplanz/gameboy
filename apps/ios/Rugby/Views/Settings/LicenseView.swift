//
//  LicenseView.swift
//  Rugby
//
//  Created by Zakhary Kaplan on 2024-06-21.
//

import SwiftUI

struct LicenseView: View {
    @ObservedObject var license: License

    var body: some View {
        ScrollView([.horizontal, .vertical]) {
            Text(license.text)
                .monospaced()
                .font(.caption)
        }
        .padding()
    }
}

class License: ObservableObject {
    @Published var text = String()

    init(path: String) {
        load(path: path)
    }

    func load(path: String) {
        if let path = Bundle.main.path(forResource: path, ofType: nil) {
            do {
                let text = try String(contentsOfFile: path)
                DispatchQueue.main.async {
                    self.text = text
                }
            } catch let error as NSError {
                print(error.localizedDescription)
            }
        }
    }
}

#Preview {
    LicenseView(license: License(path: "LICENSE-MIT"))
}
