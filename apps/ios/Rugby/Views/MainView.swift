//
//  MainView.swift
//  Rugby
//
//  Created by Zakhary Kaplan on 2024-06-20.
//

import SwiftUI

struct MainView: View {
    @State private var showSettings = false

    var body: some View {
        NavigationStack {
            LibraryView()
                .fullScreenCover(isPresented: $showSettings) {
                    NavigationStack {
                        SettingsView()
                            .toolbar {
                                Button {
                                    showSettings.toggle()
                                } label: {
                                    Text("Done").bold()
                                }
                            }
                    }
                }
                .toolbar {
                    Button {
                        showSettings.toggle()
                    } label: {
                        Label("Settings", systemImage: "gearshape")
                    }
                }
        }
    }
}

#Preview {
    MainView()
        .environment(Library())
}
