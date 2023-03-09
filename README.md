# Bad Apple Particle

Bad apple played with weird particle shit

## Setup

Use ffmpeg to extract the audio and frames from the bad apple video:
```
$ ffmpeg -i bad_apple.mp4 assets/frames/out%04d.png
$ ffmpeg -i bad_apple.mp4 -vn -acodec libvorbis assets/bad_apple.ogg
```

Build the project:
```
$ cargo build --release
```
