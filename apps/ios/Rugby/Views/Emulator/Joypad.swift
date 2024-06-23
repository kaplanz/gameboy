//
//  Joypad.swift
//  Rugby
//
//  Created by Zakhary Kaplan on 2024-06-23.
//

import SwiftUI

struct Joypad: View {
    var body: some View {
        VStack {
            HStack {
                DPad()
                Spacer()
                Keys()
            }
            Special()
        }
        .padding()
    }
}

#Preview {
    Joypad()
}

private struct DPad: View {
    var body: some View {
        Grid(horizontalSpacing: -5, verticalSpacing: 0) {
            GridRow {
                Spacer()
                Pad()
                    .rotationEffect(.degrees(-90))
                Spacer()
            }
            GridRow {
                Pad()
                    .rotationEffect(.degrees(0))
                Rectangle()
                    .frame(width: 40, height: 40)
                    .overlay {
                        Circle()
                            .fill(.gray.gradient.blendMode(.hardLight))
                            .frame(height: 30)
                    }
                Pad()
                    .rotationEffect(.degrees(180))
            }
            GridRow {
                Spacer()
                Pad()
                    .rotationEffect(.degrees(90))
                Spacer()
            }
        }
        .frame(width: 130, height: 130)
    }
}

private struct Pad: View {
    @State private var pressed = false

    var body: some View {
        Button {
            pressed = true
        } label: {
            HStack {
                HStack(spacing: 2.5) {
                    ridge
                    ridge
                    ridge
                }
            }
            .frame(width: 45, height: 40)
            .background(.black)
            .clipShape(.rect(cornerRadius: 5.0))
        }
    }

    var ridge: some View {
        RoundedRectangle(cornerRadius: 5.0)
            .fill(.gray.blendMode(.hardLight))
            .frame(width: 10, height: 30)
    }
}

private struct Keys: View {
    var body: some View {
        HStack {
            Round(label: "B")
            Spacer()
                .frame(maxWidth: 20)
            Round(label: "A")
        }
        .rotationEffect(.degrees(-20))
        .padding()
    }
}

private struct Round: View {
    @State private var pressed = false

    var label: String

    var body: some View {
        VStack {
            Button {
                pressed = true
            } label: {
                Circle()
                    .stroke(.black, lineWidth: 7.5)
                    .fill(Color.button.gradient)
                    .frame(height: 60)
            }
            Text(label)
                .foregroundStyle(Color.navy)
                .font(.system(.callout, design: .rounded).weight(.heavy))
        }
    }
}

private struct Special: View {
    var body: some View {
        HStack {
            Pill(label: "SELECT")
                .rotationEffect(.degrees(-20))
            Spacer()
                .frame(width: 20)
            Pill(label: "START")
                .rotationEffect(.degrees(-20))
        }
        .padding()
    }
}

private struct Pill: View {
    @State private var pressed = false

    var label: String

    var body: some View {
        VStack {
            Button {
                pressed = true
            } label: {
                RoundedRectangle(cornerRadius: 25.0)
                    .stroke(.black, lineWidth: 5)
                    .fill(Color.cover.gradient)
                    .frame(width: 60, height: 20)
            }
            Text(label)
                .foregroundStyle(Color.navy)
                .font(.system(.callout, design: .rounded).weight(.heavy))
        }
    }
}
