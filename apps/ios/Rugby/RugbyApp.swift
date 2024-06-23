//
//  RugbyApp.swift
//  Rugby
//
//  Created by Zakhary Kaplan on 2024-06-20.
//

import SwiftUI

@main
struct RugbyApp: App {
    @State private var lib = Library()

    var body: some Scene {
        WindowGroup {
            MainView()
                .environment(self.lib)
        }
    }
}
