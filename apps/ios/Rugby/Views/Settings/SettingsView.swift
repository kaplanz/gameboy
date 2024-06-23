//
//  SettingsView.swift
//  Rugby
//
//  Created by Zakhary Kaplan on 2024-06-20.
//

import SwiftUI

struct SettingsView: View {
    @Environment(\.openURL) private var openURL

    @State private var pal = Palette.mono
    @State private var spd = Speed.actual

    var body: some View {
        Form {
            // Application
            Section {
                Picker("Palette", selection: $pal) {
                    ForEach(Palette.allCases) { pal in
                        Text(pal.description)
                    }
                }
                .pickerStyle(.navigationLink)
                Picker("Speed", selection: $spd) {
                    ForEach(Speed.allCases) { spd in
                        Text(spd.description)
                    }
                }
            } header: {
                Text("Application")
            }
            // About
            Section {
                Link(
                    destination: URL(
                        string: "https://github.com/kaplanz/rugby"
                    )!
                ) {
                    Label("Website", systemImage: "globe")
                }
                Menu {
                    NavigationLink {
                        LicenseView(license: License(path: "LICENSE-MIT"))
                    } label: {
                        Label("MIT", systemImage: "building.columns")
                    }
                    NavigationLink {
                        LicenseView(license: License(path: "LICENSE-APACHE"))
                    } label: {
                        Label("Apache-2.0", systemImage: "bird")
                    }
                } label: {
                    Label("License", systemImage: "doc.text")
                }
            } header: {
                Text("About")
            }
        }
        .navigationTitle("Settings")
    }
}

enum Palette: String, CaseIterable, CustomStringConvertible, Identifiable {
    case legacy
    case mono
    // impl CustomStringConvertible
    var description: String {
        rawValue.capitalized
    }

    // impl Identifiable
    var id: Self { self }
}

enum Speed: Float, CaseIterable, CustomStringConvertible, Identifiable {
    case half = 0.5
    case actual = 1.0
    case double = 2.0
    case turbo = 0.0
    // impl CustomStringConvertible
    var description: String {
        switch self {
        case .turbo:
            return "Turbo"
        default:
            return rawValue.formatted(.percent)
        }
    }

    // impl Identifiable
    var id: Self { self }
}

#Preview {
    NavigationStack {
        SettingsView()
    }
}
