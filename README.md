<p align="center">
  <img src="https://github.com/user-attachments/assets/5aa36739-76c8-4a34-9a9b-7e9272927f22" alt="MechvibesDZN Logo"/>
</p>

# MechvibesDZN

[![Windows Build](https://github.com/DznDani/mechvibes-dzn/actions/workflows/windows-build.yml/badge.svg)](https://github.com/DznDani/mechvibes-dzn/actions/workflows/windows-build.yml)

**A fun and practical way to bring your favorite sounds anywhere!**

MechvibesDZN lets you play any sound when you type or click. Use it for education, presentations, gaming, or just for fun.

## ⚠️ This is a personal fork ⚠️

I just needed some fixes that annoyed me and i will add some features i find useful for my use case, if u want to use it, don't expect god level programming.

### Fixes and features added
- Implemented Github Actions workflow (only for windows builds)
- Fixed default output device refresh
- Added single and double click from tray to open the app

## Features

-   Play sounds on every keystroke (keydown/keyup) and mouse click (press/release)
-   Works with education, business, gaming, and accessibility needs
-   Global hotkey toggle (`Ctrl+Alt+M`)
-   System tray integration
-   Custom soundpack support
-   Multiple themes available
-   Logo and background customizations

## Installation

1. Download from [Releases](https://github.com/dzndani/mechvibes-dzn/releases)
2. Run installer
3. Select soundpacks
4. Enjoy the sounds or playing with customizations

## Use cases

**Education** - Musical scales, animal sounds, language learning

**Business** - Professional typewriter sounds, meeting-friendly modes

**Gaming** - Retro arcade sounds, custom sound effects

**Accessibility** - Audio feedback for visually impaired users

## Creating soundpacks

1. Record audio files (OGG, WAV, MP3)
2. Create config.json mapping keys to sounds
3. Drag and drop folder into app

```
Piano pack/
├── config.json
├── piano.ogg
└── icon.png
```

## Troubleshooting

**No sounds?** Check if muted (`Ctrl+Alt+M`), soundpack selected, system volume

**Hotkey not working?** Run as administrator, check for conflicts

**Soundpack won't load?** Verify config.json syntax, supported audio formats

## License

MIT License - do whatever you want with it.

Report bugs or request features via GitHub Issues.
