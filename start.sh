#!/bin/bash

echo "🎮 QCXIS Game Server - Quick Start"
echo "=================================="
echo ""

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed!"
    echo "📥 Install Rust from: https://rustup.rs/"
    echo ""
    echo "Run this command:"
    echo "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "✅ Rust is installed: $(rustc --version)"
echo ""

# Check if .env exists
if [ ! -f ".env" ]; then
    echo "📝 Creating .env file..."
    cp .env.example .env
    echo "⚠️  Please edit .env and set JWT_SECRET to match your Laravel APP_KEY"
    echo ""
    read -p "Press Enter after you've updated .env..."
fi

echo "🔨 Building game server..."
cargo build --release

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Build successful!"
    echo ""
    echo "🚀 Starting game server..."
    echo ""
    ./target/release/qcxis-game-server
else
    echo ""
    echo "❌ Build failed!"
    echo "Please check the error messages above."
    exit 1
fi
