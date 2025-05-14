# YouTube Chat Overlay

A desktop application that creates a customizable overlay for YouTube livestream chats. This tool allows streamers and viewers to display YouTube chat on top of other applications with customizable appearance settings.

## Features

- Display YouTube livestream chat as an overlay on your screen
- Customizable appearance:
  - Font size adjustment
  - Custom fonts (via Google Fonts)
  - Background color and opacity
  - Text colors for messages and author names
- Window locking (click-through and always-on-top mode)
- System tray icon for quick access to settings
- Toggle between "Top Chat" and "Live Chat" modes
- Persistent settings between sessions

## Prerequisites

- [Node.js](https://nodejs.org/) (v16 or newer)
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

## Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/supaflyENJOY/youtube-chat-overlay.git
   cd youtube-chat-overlay
   ```

2. Install dependencies:

   ```bash
   npm install
   ```

3. Build and run the application in development mode:

   ```bash
   npm run tauri dev
   ```

4. For production build:
   ```bash
   npm run tauri build
   ```
   The built application will be available in the `src-tauri/target/release` directory.

## Usage

1. Launch the application
2. Enter a YouTube livestream URL (e.g., `https://www.youtube.com/watch?v=jfKfPfyJRdk` or `https://www.youtube.com/live/jfKfPfyJRdk`)
3. Click "Open" to load the chat
4. Use the system tray icon to:
   - Lock/unlock the window (makes it click-through and always on top)
   - Open settings
   - Quit the application

## Settings

Access the settings window through the system tray icon to customize:

- **Chat Type**: Choose between "Full Chat" (all messages) or "Top Chat" (filtered messages)
- **Font Size**: Adjust the size of chat text
- **Message Font**: Set custom font for chat messages (using Google Fonts)
- **Author Font**: Set custom font for author names (using Google Fonts)
- **Background Color**: Change the background color and opacity
- **Message Color**: Set the color for chat messages
- **Author Color**: Set the color for author names

## Development

This application is built with:

- [Tauri](https://tauri.app/) - Framework for building desktop applications with web technologies
- [Rust](https://www.rust-lang.org/) - Backend language
- HTML/CSS/JavaScript - Frontend
- [Tera](https://tera.netlify.app/) - Templating engine for CSS injection
