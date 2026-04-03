"""
Download Bad Apple video and convert to ASCII frames for the campfire player.

Usage: python video2ascii.py [--width 80] [--height 40] [--fps 15]
Output: assets/badapple.txt
"""

import argparse
import os
import sys
import tempfile

def download_video(url, output_path):
    """Download video using yt-dlp."""
    import yt_dlp
    ydl_opts = {
        'format': 'worst[ext=mp4]/worst',  # smallest quality is fine
        'outtmpl': output_path,
        'quiet': False,
        'no_warnings': True,
    }
    with yt_dlp.YoutubeDL(ydl_opts) as ydl:
        ydl.download([url])

def frame_to_ascii(frame_gray, ascii_w, ascii_h):
    """Convert a grayscale frame to ASCII art string."""
    import cv2
    resized = cv2.resize(frame_gray, (ascii_w, ascii_h))
    # Characters from dark to light
    chars = " .:-=+*#%@"
    lines = []
    for row in range(ascii_h):
        line = []
        for col in range(ascii_w):
            pixel = resized[row, col]
            idx = int(pixel / 256 * len(chars))
            idx = min(idx, len(chars) - 1)
            line.append(chars[idx])
        lines.append("".join(line).rstrip())
    return lines

def convert_video(video_path, output_path, ascii_w, ascii_h, target_fps):
    """Read video, sample at target_fps, convert each frame to ASCII."""
    import cv2
    cap = cv2.VideoCapture(video_path)
    if not cap.isOpened():
        print(f"Error: cannot open {video_path}")
        sys.exit(1)

    video_fps = cap.get(cv2.CAP_PROP_FPS)
    total_frames = int(cap.get(cv2.CAP_PROP_FRAME_COUNT))
    duration = total_frames / video_fps if video_fps > 0 else 0

    print(f"Video: {total_frames} frames, {video_fps:.1f} fps, {duration:.1f}s")
    print(f"Output: {ascii_w}x{ascii_h} @ {target_fps} fps")

    # Sample every N-th frame
    frame_interval = video_fps / target_fps
    expected_output = int(duration * target_fps)
    print(f"Expected output frames: ~{expected_output}")

    frame_idx = 0
    next_sample = 0.0
    written = 0

    with open(output_path, 'w', encoding='utf-8') as f:
        while True:
            ret, frame = cap.read()
            if not ret:
                break

            if frame_idx >= next_sample:
                gray = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)
                lines = frame_to_ascii(gray, ascii_w, ascii_h)

                if written > 0:
                    f.write("---\n")
                for line in lines:
                    f.write(line + "\n")
                written += 1

                next_sample += frame_interval

                if written % 100 == 0:
                    pct = frame_idx / total_frames * 100
                    print(f"  {written} frames written ({pct:.0f}%)")

            frame_idx += 1

    cap.release()
    print(f"Done: {written} frames written to {output_path}")

def main():
    parser = argparse.ArgumentParser(description="Convert Bad Apple video to ASCII frames")
    parser.add_argument("--width", type=int, default=80, help="ASCII art width in chars")
    parser.add_argument("--height", type=int, default=40, help="ASCII art height in chars")
    parser.add_argument("--fps", type=int, default=15, help="Target playback FPS")
    parser.add_argument("--video", type=str, default=None, help="Path to existing video file (skip download)")
    parser.add_argument("--url", type=str, default="https://www.youtube.com/watch?v=FtutLA63Cp8",
                        help="YouTube URL to download")
    args = parser.parse_args()

    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_dir = os.path.dirname(script_dir)
    output_path = os.path.join(project_dir, "assets", "badapple.txt")
    os.makedirs(os.path.dirname(output_path), exist_ok=True)

    if args.video and os.path.exists(args.video):
        video_path = args.video
        print(f"Using existing video: {video_path}")
    else:
        video_path = os.path.join(tempfile.gettempdir(), "badapple.mp4")
        if os.path.exists(video_path):
            print(f"Video already downloaded: {video_path}")
        else:
            print(f"Downloading from {args.url} ...")
            download_video(args.url, video_path)

    convert_video(video_path, output_path, args.width, args.height, args.fps)

if __name__ == "__main__":
    main()
