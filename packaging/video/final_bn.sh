#!/bin/sh
set -e
cd "$(dirname "$0")"
export CG_LANG=bn
TOTAL=$(python -c "import sys;sys.path.insert(0,'.');import render as R;print(round(R.TOTAL,3))" | tail -n 1)
A=$(python -c "print(round($TOTAL/3))"); B=$(python -c "print(round(2*$TOTAL/3))")
python -c "
import sys;sys.path.insert(0,'.');sys.stdout.reconfigure(encoding='utf-8')
import render as R
R.build_audio('out_bn/audio.wav');R.build_srt('bn','out_bn/bn.srt');R.build_srt('en','out_bn/en.srt');R.chapters_txt('out_bn/chapters.txt')"
rm -f out_bn/video_*.mp4
python render.py video 0 $A > b1.log 2>&1 &
python render.py video $A $B > b2.log 2>&1 &
python render.py video $B $TOTAL > b3.log 2>&1
wait
grep -q "^done" b1.log && grep -q "^done" b2.log && grep -q "^done" b3.log
cd out_bn
ls video_*.mp4 | sort -t_ -k2 -n | sed "s/.*/file '&'/" > list.txt
ffmpeg -v error -y -f concat -safe 0 -i list.txt -c copy silent.mp4
DUR=$(ffprobe -v error -show_entries format=duration -of csv=p=0 silent.mp4)
ffmpeg -v error -y -i silent.mp4 -i audio.wav -i bn.srt -i en.srt -map 0:v -map 1:a -map 2 -map 3 -c:v copy -af "loudnorm=I=-16:TP=-1.5:LRA=11,apad" -t $DUR -c:a aac -b:a 192k -ar 48000 -ac 2 -c:s mov_text -metadata:s:s:0 language=ben -metadata:s:s:0 title=Bangla -metadata:s:s:1 language=eng -metadata:s:s:1 title=English -movflags +faststart HTL-Content-Guard-PC-Tutorial-Bangla-voice.mp4
ffmpeg -v error -y -i HTL-Content-Guard-PC-Tutorial-Bangla-voice.mp4 -map 0:v -map 0:a -vf scale=1280:720:flags=lanczos -c:v libx264 -preset slow -crf 27 -c:a aac -b:a 96k -movflags +faststart HTL-Content-Guard-PC-Tutorial-Bangla-voice-phone-preview.mp4
echo FINAL_OK $TOTAL
