#!/bin/sh
set -e
cd "$(dirname "$0")"
rm -f out/video_*.mp4
python render.py video 0 140 > r1.log 2>&1 &
python render.py video 140 280 > r2.log 2>&1 &
python render.py video 280 418.996 > r3.log 2>&1
wait
grep -q "^done" r1.log && grep -q "^done" r2.log && grep -q "^done" r3.log
cd out
printf "file 'video_0_140.mp4'\nfile 'video_140_280.mp4'\nfile 'video_280_418.mp4'\n" > list.txt
ffmpeg -v error -y -f concat -safe 0 -i list.txt -c copy silent.mp4
ffmpeg -v error -y -i silent.mp4 -i audio.wav -i HTL-Content-Guard-PC-Tutorial.en.srt -i HTL-Content-Guard-PC-Tutorial.bn.srt -map 0:v -map 1:a -map 2 -map 3 -c:v copy -af "loudnorm=I=-16:TP=-1.5:LRA=11,apad" -t 418.966 -c:a aac -b:a 192k -ar 48000 -ac 2 -c:s mov_text -metadata:s:s:0 language=eng -metadata:s:s:0 title=English -metadata:s:s:1 language=ben -metadata:s:s:1 title=Bangla -movflags +faststart HTL-Content-Guard-PC-Tutorial.mp4
ffmpeg -v error -y -i HTL-Content-Guard-PC-Tutorial.mp4 -map 0:v -map 0:a -vf "subtitles=HTL-Content-Guard-PC-Tutorial.bn.srt:force_style='FontName=Nirmala UI,FontSize=13,PrimaryColour=&H00FFFFFF,BackColour=&H90000000,BorderStyle=4,Outline=3,Shadow=0,MarginV=28,MarginL=60,MarginR=60'" -c:v libx264 -preset medium -crf 18 -pix_fmt yuv420p -c:a copy -movflags +faststart HTL-Content-Guard-PC-Tutorial-Bangla-subtitles.mp4
echo FINAL_OK
