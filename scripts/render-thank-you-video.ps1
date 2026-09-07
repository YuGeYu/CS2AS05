$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
$out = Join-Path $root 'artifacts\thank-you-video'
$ffmpeg = (Get-Command ffmpeg).Source
$font = 'msyh.ttc'
New-Item -ItemType Directory -Force -Path $out | Out-Null
Copy-Item 'C:\Windows\Fonts\msyh.ttc' (Join-Path $out $font) -Force

function Render-Clip([string]$name, [int]$duration, [string]$filter) {
  $path = Join-Path $out $name
  $filterPath = Join-Path $out "$name.filter.txt"
  [IO.File]::WriteAllText($filterPath, "[0:v]$filter[vout]", [Text.UTF8Encoding]::new($false))
  & $ffmpeg -y -f lavfi -i "color=c=0x07111f:s=1280x720:r=30:d=$duration" -filter_complex_script $filterPath -map '[vout]' -an -c:v libx264 -preset medium -crf 18 -pix_fmt yuv420p -movflags +faststart $path
  if ($LASTEXITCODE -ne 0) { throw "FFmpeg failed for $name" }
}

$fontSpec = $font
$intro = @"
drawbox=x=0:y=0:w=1280:h=720:color=0x0b2033@1:t=fill,
drawbox=x=54:y=54:w=1172:h=612:color=0x12314a@0.55:t=fill,
drawbox=x=54:y=54:w=1172:h=612:color=0x64b5ff@0.7:t=2,
drawbox=x=94:y=570:w=1092:h=2:color=0xf2c46d@0.55:t=fill,
drawtext=fontfile='$font':text='CS2 人机增强助手':fontcolor=0xf7fbff:fontsize=24:x=94:y=92,
drawtext=fontfile='$font':text='0.5.8':fontcolor=0x8ec8ff:fontsize=22:x=1090:y=94,
drawtext=fontfile='$font':text='贡献陈列馆':fontcolor=0xf2c46d:fontsize=26:x=94:y=188,
drawtext=fontfile='$font':text='幸与诸君同路':fontcolor=0xffffff:fontsize=68:x=94:y=240,
drawtext=fontfile='$font':text='感谢每一位同路人与上游作者':fontcolor=0xd8e9f7:fontsize=30:x=96:y=360,
drawtext=fontfile='$font':text='正在展示鸣谢长廊':fontcolor=0x8fb3ca:fontsize=23:x=96:y=430,
drawbox=x=96:y=516:w=1088:h=8:color=0x1f4a68@1:t=fill,
drawbox=x=96:y=516:w=800:h=8:color=0x6ed0ff@1:t=fill,
drawtext=fontfile='$font':text='感谢上游项目  ·  感谢同路人  ·  把热爱留在每一次开局':fontcolor=0xf2c46d:fontsize=24:x=96:y=600
"@ -replace "`r?`n",''
$intro = $intro.Replace("fontfile='$font'", "fontfile=$fontSpec")

$gallery = @"
drawbox=x=0:y=0:w=1280:h=720:color=0x081a25@1:t=fill,
drawbox=x=36:y=36:w=1208:h=648:color=0x102d38@0.7:t=fill,
drawbox=x=36:y=36:w=1208:h=648:color=0x6dd3c4@0.75:t=2,
drawbox=x=640:y=154:w=2:h=386:color=0xe5c88d@0.55:t=fill,
drawbox=x=660:y=222:w=330:h=250:color=0x1e4e5a@1:t=fill,
drawbox=x=680:y=242:w=290:h=210:color=0x2f6970@1:t=fill,
drawtext=fontfile='$font':text='贡献陈列馆':fontcolor=0xf0d28a:fontsize=25:x=92:y=92,
drawtext=fontfile='$font':text='众行者，共铸此间':fontcolor=0xffffff:fontsize=58:x=92:y=158,
drawtext=fontfile='$font':text='WASD / 方向键行走  ·  拖动鼠标环顾':fontcolor=0x9ed2d1:fontsize=21:x=94:y=270,
drawtext=fontfile='$font':text='中央珍藏':fontcolor=0xf0d28a:fontsize=22:x=700:y=504,
drawtext=fontfile='$font':text='唐代彩绘仕女俑':fontcolor=0xffffff:fontsize=32:x=700:y=544,
drawtext=fontfile='$font':text='盛唐风华 · 高髻宽袖与彩绘余晖':fontcolor=0xaed9d0:fontsize=20:x=700:y=596,
drawtext=fontfile='$font':text='中央展品来源  cultural-relics-museum  ·  MulanPSL-2.0':fontcolor=0x769da5:fontsize=18:x=92:y=646,
drawtext=fontfile='$font':text='一段彩蛋，一份致意':fontcolor=0xf0d28a:fontsize=25:x=94:y=358,
drawtext=fontfile='$font':text='感谢愿意一起把工具做得更好的你':fontcolor=0xffffff:fontsize=25:x=94:y=408
"@ -replace "`r?`n",''
$gallery = $gallery.Replace("fontfile='$font'", "fontfile=$fontSpec")

$end = @"
drawbox=x=0:y=0:w=1280:h=720:color=0x091522@1:t=fill,
drawbox=x=80:y=80:w=1120:h=560:color=0x112b3d@0.8:t=fill,
drawbox=x=80:y=80:w=1120:h=560:color=0xf2c46d@0.8:t=2,
drawtext=fontfile='$font':text='致每一位同路人':fontcolor=0xf2c46d:fontsize=28:x=440:y=176,
drawtext=fontfile='$font':text='谢谢你们，让这段路有了回声':fontcolor=0xffffff:fontsize=46:x=232:y=260,
drawtext=fontfile='$font':text='也感谢每一个被我们借鉴、学习、致敬的上游项目':fontcolor=0xbad8e8:fontsize=24:x=218:y=364,
drawtext=fontfile='$font':text='CS2 人机增强助手  ·  B 站感谢视频':fontcolor=0x8ec8ff:fontsize=22:x=408:y=486,
drawtext=fontfile='$font':text='愿下一局，仍然并肩':fontcolor=0x9ee6cf:fontsize=28:x=470:y=548
"@ -replace "`r?`n",''
$end = $end.Replace("fontfile='$font'", "fontfile=$fontSpec")

Push-Location $out
Render-Clip 'video1-opening-pure.mp4' 10 $intro
Render-Clip 'video2-easter-egg-pure.mp4' 12 $gallery
Render-Clip 'video3-thanks-endcard.mp4' 8 $end

$list = Join-Path $out 'concat.txt'
@("file 'video1-opening-pure.mp4'", "file 'video2-easter-egg-pure.mp4'", "file 'video3-thanks-endcard.mp4'") | Set-Content -Encoding ascii $list
& $ffmpeg -y -f concat -safe 0 -i $list -c copy -movflags +faststart (Join-Path $out 'thank-you-film.mp4')
if ($LASTEXITCODE -ne 0) { throw 'FFmpeg concat failed' }
Pop-Location

Get-ChildItem $out -Filter '*pure.mp4' | Get-FileHash -Algorithm SHA256 | Format-Table -AutoSize
Get-FileHash (Join-Path $out 'thank-you-film.mp4') -Algorithm SHA256 | Format-Table -AutoSize
