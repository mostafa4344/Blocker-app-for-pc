$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class W {
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint f);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr FindWindowEx(IntPtr p, IntPtr c, string cls, string title);
  [DllImport("user32.dll")] public static extern IntPtr SendMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] public static extern IntPtr GetDlgItem(IntPtr h, int id);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  public struct RECT { public int L, T, R, B; }
}
"@
[W]::SetProcessDPIAware() | Out-Null
$out = "C:\Users\user\AppData\Local\Temp\cgvideo\shots"
New-Item -ItemType Directory -Force $out | Out-Null
$p = Start-Process "C:\Users\user\AppData\Local\Temp\cgvideo\nsis\HTL-Content-Guard-PC-Setup.exe" -PassThru
$h = [IntPtr]::Zero
for ($i = 0; $i -lt 50 -and $h -eq [IntPtr]::Zero; $i++) {
  Start-Sleep -Milliseconds 200
  $p.Refresh(); $h = $p.MainWindowHandle
}
if ($h -eq [IntPtr]::Zero) { throw "no window" }

function Shot($name) {
  Start-Sleep -Milliseconds 900
  $r = New-Object W+RECT
  [W]::GetWindowRect($h, [ref]$r) | Out-Null
  $bmp = New-Object System.Drawing.Bitmap ($r.R - $r.L), ($r.B - $r.T)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $dc = $g.GetHdc()
  [W]::PrintWindow($h, $dc, 2) | Out-Null
  $g.ReleaseHdc($dc); $g.Dispose()
  $bmp.Save("$out\$name.png", [System.Drawing.Imaging.ImageFormat]::Png)
  $bmp.Dispose()
  Write-Output "shot $name $($r.R - $r.L)x$($r.B - $r.T)"
}
# Clicks the wizard's Next/Install/Finish button (control id 1) directly.
function Next() { [W]::SendMessage([W]::GetDlgItem($h, 1), 0x00F5, [IntPtr]::Zero, [IntPtr]::Zero) | Out-Null }

Shot "inst1_welcome"; Next
Shot "inst2_directory"; Next
Start-Sleep -Milliseconds 400
Shot "inst3_progress"
Start-Sleep -Seconds 4
Shot "inst4_finish"; Next
Start-Sleep -Seconds 1
if (-not $p.HasExited) { $p.Kill() }
Remove-Item -Recurse -Force "$env:TEMP\cg-installer-shot" -ErrorAction SilentlyContinue
