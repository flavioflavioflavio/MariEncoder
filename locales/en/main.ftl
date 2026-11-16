## Mari Encoder — English

## Tabs
tab-queue = Queue
tab-presets = Presets
tab-settings = Settings
tab-about = About

## General notices
toast-preset-missing = That preset no longer exists — pick another one from the list.
toast-create-preset-first = Create a preset before adding files.
toast-save-failed = Couldn't save the settings: { $error }
toast-menu-update-failed = Couldn't update the context menu: { $error }
toast-modern-update-failed = Couldn't update the Windows 11 menu: { $error }
toast-menu-install-failed = Couldn't install the menu: { $error }
toast-menu-modern-failed = Classic menu installed. The Windows 11 menu failed: { $error }
toast-menu-installed = Context menu installed.
toast-menu-removed = Context menu removed.
toast-menu-remove-failed = Couldn't remove the menu: { $error }
ffmpeg-missing-banner = ffmpeg not found — see Settings

## Queue
dialog-add-videos = Add videos
filter-videos = Videos
filter-all-files = All files
queue-add-files = ➕  Add files
queue-preset = Preset
queue-apply-all = Apply to all
queue-apply-all-tip = Use this preset for every file that hasn't started yet
job-remove = Remove from list
job-cancel = Cancel
job-open-folder = Open folder
job-open-folder-tip = Show the output file in Explorer
job-retry = Try again
queue-count = { $count ->
    [0] No files
    [one] 1 file
   *[other] { $count } files
}
queue-active = { $count } in progress
queue-done = { $count } done
queue-failed = { $count } failed
queue-encode-one = ▶  Encode
queue-encode-many = ▶  Encode { $count } files
queue-cancel-all = Cancel all
queue-clear-finished = Clear finished
queue-empty-title = Drag videos or folders here
queue-empty-hint = or right-click a video in Explorer → { $menu }
compact-done = Done ✔
compact-open-full = Open full window
drop-overlay = Drop to add to the queue

## Per-file status
status-ready = { $size } · ready to encode
status-queued = { $preset } · queued
status-progress = { $preset } · { $percent }%
status-speed = { $speed }x
status-eta = { $time } left
status-encoding = { $preset } · encoding…
status-done = { $preset } · { $from } → { $to }
status-error = Error: { $message }
status-unknown-error = unknown error
status-cancelled = { $preset } · cancelled

## Presets
presets-empty = No presets. Create one with “New”.
presets-new = ➕ New
presets-duplicate = Duplicate
presets-copy-name = { $name } (copy)
presets-delete = Delete
presets-move-up = Move up in the menu
presets-move-down = Move down in the menu
presets-not-in-menu = not in menu
preset-new-name = New preset

editor-general = General
editor-name = Name
editor-format = Format
editor-context-menu = Context menu
editor-show-in-menu = Show on right-click
editor-gif = GIF
editor-max-size = Maximum size
editor-max-fps = Maximum FPS
editor-video = Video
editor-codec = Codec
editor-encoder = Encoder
editor-rate-control = Rate control
editor-quality = Quality
editor-quality-hint = lower = better · H.264 18–28 · HEVC 22–30 · AV1/VP9 28–40
editor-video-bitrate = Video bitrate
editor-file-size = File size
editor-file-size-hint = approximate (single pass)
editor-speed = Speed
editor-max-resolution = Maximum resolution
editor-audio = Audio
editor-bitrate = Bitrate
editor-output = Output
editor-suffix = Name suffix
editor-folder = Folder
editor-folder-hint = Same folder as the original
editor-output-example = video{ $suffix }.{ $ext }  ·  a subfolder name works too
editor-extra-args = Extra ffmpeg args
editor-optional = optional
editor-command = Command
browse = Browse…
original = Original

## Preset options
container-gif = Animated GIF
container-mp3 = MP3 (audio only)
container-m4a = M4A (audio only)
container-wav = WAV (audio only)
codec-copy = Copy (no re-encode)
hw-windows = Windows (automatic)
hw-nvidia = NVIDIA GPU (NVENC)
hw-intel = Intel GPU (Quick Sync)
hw-amd = AMD GPU (AMF)
rate-quality = Constant quality (CRF)
rate-bitrate = Fixed bitrate
rate-target-size = Target file size
speed-fast = Fast
speed-medium = Balanced
speed-slow = Slow (better compression)
audio-pcm = PCM (uncompressed)
audio-none = No audio

## Settings
settings-menu = Windows context menu
settings-menu-store = ● Active — installed with the app from the Microsoft Store
settings-menu-store-hint = Right-click a video → { $menu }, with one item per preset. On Windows 11 it shows in the main menu; Windows decides its position.
settings-menu-both = ● Installed — Windows 11 menu and classic menu
settings-menu-modern-only = ● Installed — Windows 11 menu only
settings-menu-classic-only = ● Installed — classic menu only
settings-menu-not-installed = ● Not installed
settings-install = Install
settings-update = Update
settings-remove = Remove
settings-menu-hint = Right-click a video → { $menu }, with one item per preset. In the classic menu (“Show more options” or Shift + right-click) it stays at the top; in the Windows 11 main menu the system decides its position. No administrator rights needed.
settings-modern-needs-devmode = The Windows 11 main menu isn't registered: without a signature it requires Developer Mode. Click Update to try again.
settings-modern-unavailable = Windows 11 main menu unavailable: it needs Windows 11 and MariEncoderMenu.dll in the app folder.
settings-extensions-store = Video extensions
settings-extensions = Extensions with the menu
settings-extensions-store-hint = Comma-separated. They define what is picked up when you drag a folder onto the window.
settings-extensions-hint = Comma-separated. They also define what is picked up when you drag a folder.
settings-ffmpeg = ffmpeg
settings-ffmpeg-found = ● Found: { $path }
settings-ffmpeg-missing = ● Not found
settings-ffmpeg-folder = ffmpeg folder
settings-ffmpeg-auto = Automatic (next to the app or on PATH)
settings-behavior = Behavior
settings-close-when-done = Close the quick window when everything finishes without errors
settings-default-preset = Window's default preset
settings-language = Language
settings-config-path = Settings file: { $path }
lang-system = Windows language

## About
about-version = Version { $version }
about-desc = One-click video encoding, straight from the Windows right-click menu.
about-dedication = Dedicated to Mariana Lima, with all my love.
about-author-title = CREATED BY
about-author-bio = Art director, motion designer and digital product maker, with over 15 years across design, technology and communication. Founder of EvoMind, a visual AI workspace.
about-link-github = Code on GitHub
about-support-title = SUPPORT THE PROJECT
about-support-desc = { $app } is free, with no ads and no paid version. If it helps you, consider supporting it.
about-donate = ♥ Donate with PayPal
about-updates-title = UPDATES
about-updates-store = Updates arrive automatically through the Microsoft Store.
about-updates-github = See releases on GitHub
about-licenses-title = LICENSES
about-licenses-desc = Videos are converted by FFmpeg (LGPL v3), which ships with the app as a separate program. H.264, HEVC and AAC use the encoders built into Windows or your graphics card.
about-ffmpeg-source = Source code of the included FFmpeg
about-license = GPLv3 license  •  © 2026 Flávio Takemoto

## Encoding errors
err-ffmpeg-start = Couldn't start ffmpeg: { $error }
err-ffmpeg-failed = ffmpeg finished with an error.
err-ffmpeg-missing = ffmpeg not found. Set its folder in Settings.
err-input-missing = Input file not found.
err-output-folder = Couldn't create the output folder: { $error }
err-hevc-needs-gpu = HEVC through Windows needs a graphics card with an HEVC encoder — use H.264 or AV1.
err-no-encoder = No encoder available for this preset.
note-gpu-fallback = GPU unavailable — using the Windows encoder
err-devmode = { $error } (unsigned, registration requires Windows Developer Mode)
err-exe-dir = executable folder not found
err-powershell = couldn't run PowerShell: { $error }
