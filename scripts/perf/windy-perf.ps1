# Windows-only harness that drives the REAL windy.exe: start-up, idle memory and CPU, hotkey-to-popup latency,
# screenshots of the app's own windows, and a WebView2 debug-port probe. Windows PowerShell 5.1 or later.
#
# SIDE EFFECTS while it runs: it injects Ctrl+Alt+V and other keys, moves the mouse and clicks, shows a coloured
# backdrop window, and (screenshots mode) replaces the clipboard text with synthetic strings, then restores your text.
# Do not use the PC while it runs. Read scripts/perf/README.md first.
param(
  [Parameter(Mandatory = $true)][ValidateSet("startup-ram", "latency", "screenshots", "debug-probe")][string]$Mode,
  [string]$Exe = "",
  [string]$OutDir = "",
  [int]$StartRuns = 8,
  [int]$Trials = 40,
  [int]$IdleSeconds = 60,
  [int]$Cycles = 30
)
$ErrorActionPreference = "Stop"

# $PSScriptRoot is empty inside a param() default on Windows PowerShell 5.1, so the defaults are resolved here.
$script:Root = $PSScriptRoot
if (-not $script:Root) { $script:Root = Split-Path -Parent $MyInvocation.MyCommand.Path }
if (-not $Exe) {
  if ($env:CARGO_TARGET_DIR) { $Exe = Join-Path $env:CARGO_TARGET_DIR "release\windy.exe" }
  else { $Exe = Join-Path $script:Root "..\..\target\release\windy.exe" }
}
if (-not $OutDir) { $OutDir = Join-Path $script:Root "..\..\test-results\perf" }
$null = New-Item -ItemType Directory -Force -Path $OutDir

Add-Type -TypeDefinition @"
using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;

public static class W {
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc p, IntPtr l);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  [DllImport("user32.dll")] static extern void mouse_event(uint flags, int dx, int dy, uint data, UIntPtr extra);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] static extern bool GetCursorPos(out POINT p);
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr hwnd, IntPtr hdc, uint flags);
  [DllImport("user32.dll")] static extern IntPtr GetDC(IntPtr h);
  [DllImport("user32.dll")] static extern int ReleaseDC(IntPtr h, IntPtr dc);
  [DllImport("gdi32.dll")] static extern uint GetPixel(IntPtr dc, int x, int y);
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }

  public static IntPtr FindTopLevel(uint pid, string title) {
    IntPtr found = IntPtr.Zero;
    EnumWindows(delegate(IntPtr h, IntPtr l) {
      uint p; GetWindowThreadProcessId(h, out p);
      if (p != pid) return true;
      StringBuilder sb = new StringBuilder(256); GetWindowText(h, sb, 256);
      if (sb.ToString() == title) { found = h; return false; }
      return true;
    }, IntPtr.Zero);
    return found;
  }
  public static int[] Rect(IntPtr h) { RECT r; GetWindowRect(h, out r); return new int[] { r.L, r.T, r.R, r.B }; }
  public static int[] Cursor() { POINT p; GetCursorPos(out p); return new int[] { p.X, p.Y }; }
  public static uint Pixel(int x, int y) { IntPtr dc = GetDC(IntPtr.Zero); uint c = GetPixel(dc, x, y); ReleaseDC(IntPtr.Zero, dc); return c; }

  const byte CTRL = 0x11, ALT = 0x12, V = 0x56; const uint UP = 2;

  // Press and release Ctrl+Alt+V (the default popup hotkey).
  public static void Hotkey() {
    keybd_event(CTRL, 0, 0, UIntPtr.Zero); keybd_event(ALT, 0, 0, UIntPtr.Zero); keybd_event(V, 0, 0, UIntPtr.Zero);
    Thread.Sleep(30);
    keybd_event(V, 0, UP, UIntPtr.Zero); keybd_event(ALT, 0, UP, UIntPtr.Zero); keybd_event(CTRL, 0, UP, UIntPtr.Zero);
  }
  public static void Click(int x, int y) {
    SetCursorPos(x, y); Thread.Sleep(80);
    mouse_event(2, 0, 0, 0, UIntPtr.Zero); Thread.Sleep(40); mouse_event(4, 0, 0, 0, UIntPtr.Zero);
  }

  // Injects the hotkey and times the result. Time zero: just before the V key-down. Returns { windowVisibleMs,
  // firstPaintMs } (-1 = not reached within timeoutMs). Paint is measured only when showing with cx >= 0: the first time
  // the pixel at (cx,cy) differs from `pre` (the solid colour of the backdrop behind the popup).
  public static double[] Toggle(IntPtr h, bool wantVisible, int cx, int cy, uint pre, int timeoutMs) {
    IntPtr dc = GetDC(IntPtr.Zero);
    double vis = -1, paint = -1, freq = Stopwatch.Frequency;
    keybd_event(CTRL, 0, 0, UIntPtr.Zero); keybd_event(ALT, 0, 0, UIntPtr.Zero);
    long t0 = Stopwatch.GetTimestamp();
    keybd_event(V, 0, 0, UIntPtr.Zero);
    long limit = t0 + (long)(freq * timeoutMs / 1000.0);
    bool measurePaint = wantVisible && cx >= 0;
    while (true) {
      long now = Stopwatch.GetTimestamp();
      if (now > limit) break;
      if (vis < 0) {
        if (IsWindowVisible(h) == wantVisible) { vis = (now - t0) * 1000.0 / freq; if (!measurePaint) break; }
      } else if (GetPixel(dc, cx, cy) != pre) {
        paint = (Stopwatch.GetTimestamp() - t0) * 1000.0 / freq; break;
      }
    }
    keybd_event(V, 0, UP, UIntPtr.Zero); keybd_event(ALT, 0, UP, UIntPtr.Zero); keybd_event(CTRL, 0, UP, UIntPtr.Zero);
    ReleaseDC(IntPtr.Zero, dc);
    return new double[] { vis, paint };
  }
}
"@

[void][W]::SetProcessDPIAware()
Add-Type -AssemblyName System.Windows.Forms, System.Drawing

if (-not (Test-Path $Exe)) { throw "missing $Exe (build it with: cargo build -p windy --release)" }
if (Get-Process -Name windy -ErrorAction SilentlyContinue) { throw "windy.exe is already running; quit it first (it is a single-instance app)." }

$script:ErrFile = Join-Path $OutDir "stderr.txt"
$script:OutFile = Join-Path $OutDir "stdout.txt"
$script:DataMarker = "app.windy.clipboard"   # the app's WebView2 user-data folder name

# ---------------------------------------------------------------- helpers

function Pump([int]$n = 4) { 1..$n | ForEach-Object { [System.Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 30 } }

function Stats($a) {
  $v = @($a | Where-Object { $_ -ge 0 } | Sort-Object)
  if ($v.Count -eq 0) { return @{ n = 0 } }
  if ($v.Count % 2) { $med = $v[($v.Count - 1) / 2] } else { $med = ($v[$v.Count / 2 - 1] + $v[$v.Count / 2]) / 2 }
  $i95 = [Math]::Min($v.Count - 1, [int][Math]::Ceiling(0.95 * $v.Count) - 1)
  @{ n = $v.Count; min = [Math]::Round($v[0], 1); median = [Math]::Round($med, 1); p95 = [Math]::Round($v[$i95], 1)
     max = [Math]::Round($v[-1], 1); mean = [Math]::Round(($v | Measure-Object -Average).Average, 1) }
}

function Wait-Visible($hwnd, [bool]$want, [int]$ms) {
  $sw = [Diagnostics.Stopwatch]::StartNew()
  while ($sw.ElapsedMilliseconds -lt $ms) { if ([W]::IsWindowVisible($hwnd) -eq $want) { return $true }; Start-Sleep -Milliseconds 10 }
  return $false
}

# A solid-colour window behind the popup: it receives the hotkey keystrokes if the app is not ready yet (so they cannot
# reach whatever you were doing) and gives first-paint detection a known colour to compare against.
function New-Backdrop([System.Drawing.Color]$color) {
  $f = New-Object System.Windows.Forms.Form
  $f.FormBorderStyle = 'None'; $f.StartPosition = 'Manual'; $f.ShowInTaskbar = $false; $f.TopMost = $true; $f.BackColor = $color
  $f.Location = New-Object System.Drawing.Point(100, 50); $f.Size = New-Object System.Drawing.Size(300, 300)
  $f.Show(); $f.Activate(); Pump
  $f
}

function Start-App([bool]$waitForHotkey) {
  $sw = [Diagnostics.Stopwatch]::StartNew()
  $proc = Start-Process -FilePath $Exe -PassThru -WindowStyle Hidden -RedirectStandardError $script:ErrFile -RedirectStandardOutput $script:OutFile
  $hwnd = [IntPtr]::Zero
  while ($hwnd -eq [IntPtr]::Zero -and $sw.ElapsedMilliseconds -lt 20000) { $hwnd = [W]::FindTopLevel([uint32]$proc.Id, "Windy") }
  $windowMs = $sw.Elapsed.TotalMilliseconds
  $readyMs = -1
  if ($waitForHotkey) {
    # The popup's webview is created before the hotkey is registered, so retry until the hotkey opens the popup.
    [void][W]::SetCursorPos(150, 100)
    while ($sw.ElapsedMilliseconds -lt 20000) {
      $r = [W]::Toggle($hwnd, $true, -1, -1, 0, 80)
      if ($r[0] -ge 0 -or [W]::IsWindowVisible($hwnd)) { $readyMs = $sw.Elapsed.TotalMilliseconds; break }
    }
    if ([W]::IsWindowVisible($hwnd)) { [void][W]::Toggle($hwnd, $false, -1, -1, 0, 800) }
  } else {
    Start-Sleep -Seconds 5   # generous: the hotkey works about 0.7 to 2.7 s after launch on the reference machine
  }
  [pscustomobject]@{ Proc = $proc; Hwnd = $hwnd; WindowMs = $windowMs; ReadyMs = $readyMs }
}

function Stop-App($app) {
  Stop-Process -Id $app.Proc.Id -Force -ErrorAction SilentlyContinue
  $app.Proc.WaitForExit(5000) | Out-Null
  Start-Sleep -Milliseconds 1500
}

function Get-AppProcs([int]$rootPid) {
  $all = @(Get-CimInstance Win32_Process)
  $ids = New-Object 'System.Collections.Generic.HashSet[int]'
  $q = New-Object 'System.Collections.Generic.Queue[int]'
  [void]$ids.Add($rootPid); $q.Enqueue($rootPid)
  while ($q.Count -gt 0) {
    $cur = $q.Dequeue()
    foreach ($c in ($all | Where-Object { $_.ParentProcessId -eq $cur })) { if ($ids.Add([int]$c.ProcessId)) { $q.Enqueue([int]$c.ProcessId) } }
  }
  foreach ($c in ($all | Where-Object { $_.Name -eq "msedgewebview2.exe" -and $_.CommandLine -like "*$($script:DataMarker)*" })) { [void]$ids.Add([int]$c.ProcessId) }
  $ids
}

function Get-RamSample([int]$rootPid) {
  $rows = @()
  foreach ($id in (Get-AppProcs $rootPid)) {
    $perf = Get-CimInstance Win32_PerfFormattedData_PerfProc_Process -Filter "IDProcess=$id" -ErrorAction SilentlyContinue
    if ($perf) {
      $rows += [pscustomobject]@{ Name = $perf.Name; Pid = $id
        PrivateWsMB = [Math]::Round($perf.WorkingSetPrivate / 1MB, 1); WorkingSetMB = [Math]::Round($perf.WorkingSet / 1MB, 1) }
    }
  }
  [pscustomobject]@{
    Processes = $rows; Count = $rows.Count
    TotalPrivateWsMB = [Math]::Round(($rows | Measure-Object PrivateWsMB -Sum).Sum, 1)
    TotalWorkingSetMB = [Math]::Round(($rows | Measure-Object WorkingSetMB -Sum).Sum, 1)
    RustProcessPrivateWsMB = ($rows | Where-Object { $_.Pid -eq $rootPid }).PrivateWsMB
  }
}

function Get-CpuSeconds([int]$rootPid) {
  $t = @{}
  foreach ($id in (Get-AppProcs $rootPid)) { $pr = Get-Process -Id $id -ErrorAction SilentlyContinue; if ($pr) { $t[$id] = $pr.TotalProcessorTime.TotalSeconds } }
  $t
}

function Get-MachineInfo {
  $os = Get-CimInstance Win32_OperatingSystem
  $wv = (Get-ItemProperty 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}' -ErrorAction SilentlyContinue).pv
  [ordered]@{
    cpu = (Get-CimInstance Win32_Processor | Select-Object -First 1).Name.Trim()
    logical_cores = [Environment]::ProcessorCount
    ram_gb = [Math]::Round($os.TotalVisibleMemorySize / 1MB, 1)
    os = "$($os.Caption) build $($os.BuildNumber)"
    webview2_runtime = $wv
    exe = $Exe
    exe_size_mb = [Math]::Round((Get-Item $Exe).Length / 1MB, 2)
    measured_at = (Get-Date).ToString("s")
  }
}

function Save-Json($obj, [string]$name) {
  $path = Join-Path $OutDir $name
  $obj | ConvertTo-Json -Depth 6 | Set-Content -Path $path -Encoding UTF8
  $obj | ConvertTo-Json -Depth 6
  "written: $path"
}

# PrintWindow makes the window render itself, so nothing else on screen can appear in the image.
function Grab($hwnd, $name) {
  $rc = [W]::Rect($hwnd); $w = $rc[2] - $rc[0]; $ht = $rc[3] - $rc[1]
  $bmp = New-Object System.Drawing.Bitmap $w, $ht
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $hdc = $g.GetHdc(); $ok = [W]::PrintWindow($hwnd, $hdc, 2); $g.ReleaseHdc($hdc); $g.Dispose()
  $nonWhite = 0; $total = 0
  for ($yy = 60; $yy -lt ($ht - 10); $yy += 12) { for ($xx = 10; $xx -lt ($w - 10); $xx += 12) {
    $c = $bmp.GetPixel($xx, $yy); $total++
    if ($c.R -lt 240 -or $c.G -lt 240 -or $c.B -lt 240) { $nonWhite++ }
  } }
  if ($name) { $bmp.Save((Join-Path $script:ShotDir $name), [System.Drawing.Imaging.ImageFormat]::Png) }
  $bmp.Dispose()
  [pscustomobject]@{ Ok = $ok; Width = $w; Height = $ht; NonWhitePct = [Math]::Round(100.0 * $nonWhite / [Math]::Max(1, $total), 1) }
}

function Save-Rect($rc, [string]$path) {
  $w = $rc[2] - $rc[0]; $ht = $rc[3] - $rc[1]
  $bmp = New-Object System.Drawing.Bitmap $w, $ht
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($rc[0], $rc[1], 0, 0, (New-Object System.Drawing.Size $w, $ht))
  $bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png); $g.Dispose(); $bmp.Dispose()
}

function Send-Keys([string]$keys) {
  if ([W]::GetForegroundWindow() -ne $script:HwndPop) { throw "popup is not the foreground window; refusing to send keys" }
  [System.Windows.Forms.SendKeys]::SendWait($keys); Start-Sleep -Milliseconds 400
}

function Show-At([int]$px, [int]$py) {
  for ($try = 0; $try -lt 4; $try++) {
    if ([W]::IsWindowVisible($script:HwndPop)) { [W]::Hotkey(); [void](Wait-Visible $script:HwndPop $false 1500); Start-Sleep -Milliseconds 300 }
    [void][W]::SetCursorPos($px, $py); Start-Sleep -Milliseconds 100
    [W]::Hotkey()
    if (-not (Wait-Visible $script:HwndPop $true 2000)) { continue }
    Start-Sleep -Milliseconds 800
    $rc = [W]::Rect($script:HwndPop)
    if ([Math]::Abs($rc[0] - $px) -le 10 -and [Math]::Abs($rc[1] - $py) -le 10) { return $rc }
  }
  throw "could not show the popup at the expected position"
}

# Never click blind: the popup hides itself when it loses focus.
function Safe-Click($rc, [int]$dx, [int]$dy) {
  if (-not [W]::IsWindowVisible($script:HwndPop) -or [W]::GetForegroundWindow() -ne $script:HwndPop) { throw "popup not visible and focused; refusing to click" }
  [W]::Click($rc[0] + $dx, $rc[1] + $dy)
}

# ---------------------------------------------------------------- modes

function Invoke-StartupRam {
  $backdrop = New-Backdrop ([System.Drawing.Color]::FromArgb(236, 239, 244))
  $origCursor = [W]::Cursor(); $result = [ordered]@{}; $app = $null
  try {
    $win = @(); $ready = @(); $first = $null
    for ($i = 0; $i -lt $StartRuns; $i++) {
      $a = Start-App $true
      if ($i -eq 0) { $first = @{ windowMs = [Math]::Round($a.WindowMs, 1); hotkeyReadyMs = [Math]::Round($a.ReadyMs, 1) } }
      $win += $a.WindowMs; $ready += $a.ReadyMs
      Stop-App $a
    }
    $result.startup_first_run_cold = $first
    $result.startup_window_ms = Stats $win
    $result.startup_hotkey_ready_ms = Stats $ready

    $app = Start-App $true
    $rootPid = $app.Proc.Id
    Start-Sleep -Seconds $IdleSeconds
    $result.ram_idle = Get-RamSample $rootPid
    $c1 = Get-CpuSeconds $rootPid; $s1 = Get-Date
    Start-Sleep -Seconds 30
    $c2 = Get-CpuSeconds $rootPid; $s2 = Get-Date
    $used = 0.0; foreach ($k in $c1.Keys) { if ($c2.ContainsKey($k)) { $used += ($c2[$k] - $c1[$k]) } }
    $result.cpu_idle_percent_of_one_core = [Math]::Round(100.0 * $used / ($s2 - $s1).TotalSeconds, 2)

    for ($i = 0; $i -lt $Cycles; $i++) {
      [void][W]::SetCursorPos(150, 100)
      [void][W]::Toggle($app.Hwnd, $true, -1, -1, 0, 1500); Start-Sleep -Milliseconds 250
      [void][W]::Toggle($app.Hwnd, $false, -1, -1, 0, 1500); Start-Sleep -Milliseconds 350
    }
    Start-Sleep -Seconds 10
    $result.ram_after_cycles = Get-RamSample $rootPid
    $result.cycles = $Cycles
  }
  finally {
    if ($app) { Stop-App $app }
    $backdrop.Close()
    [void][W]::SetCursorPos($origCursor[0], $origCursor[1])
  }
  $result.machine = Get-MachineInfo
  Save-Json $result "startup-ram.json"
}

function Invoke-Latency {
  $shotDir = Join-Path $OutDir "first-frames"; $null = New-Item -ItemType Directory -Force -Path $shotDir
  $screen = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
  $backdrop = New-Backdrop ([System.Drawing.Color]::FromArgb(255, 0, 255))
  $magenta = [W]::Pixel(200, 150)
  if ($magenta -ne 0x00FF00FF) { throw ("backdrop colour check failed: 0x{0:X8}" -f $magenta) }
  $origCursor = [W]::Cursor(); $res = [ordered]@{}; $app = $null
  try {
    $app = Start-App $false
    $hwnd = $app.Hwnd
    [void][W]::Toggle($hwnd, $true, -1, -1, 0, 2000); Start-Sleep -Milliseconds 500
    $rc = [W]::Rect($hwnd); $popW = $rc[2] - $rc[0]; $popH = $rc[3] - $rc[1]
    [void][W]::Toggle($hwnd, $false, -1, -1, 0, 2000); Start-Sleep -Milliseconds 400
    if ($popW -lt 100) { throw "calibration failed: popup width $popW" }

    $vis = @(); $paint = @(); $discarded = 0; $noPaint = 0; $shots = 0; $log = @()
    for ($i = 0; $i -lt $Trials; $i++) {
      # The pointer decides where the popup opens, so predict its rectangle and park the backdrop exactly behind it.
      $c = [W]::Cursor()
      $x = [Math]::Max(0, [Math]::Min($c[0], $screen.Width - $popW)); $y = [Math]::Max(0, [Math]::Min($c[1], $screen.Height - $popH))
      $cx = $x + [int]($popW / 2); $cy = $y + [int]($popH / 2)
      $backdrop.Location = New-Object System.Drawing.Point(($x - 30), ($y - 30))
      $backdrop.Size = New-Object System.Drawing.Size(($popW + 60), ($popH + 60)); Pump 4
      $pre = [W]::Pixel($cx, $cy)
      if ($pre -ne $magenta) { $discarded++; if ($log.Count -lt 12) { $log += "trial ${i}: backdrop not magenta at ($cx,$cy)" }; continue }
      $r = [W]::Toggle($hwnd, $true, $cx, $cy, $pre, 1500)
      $a = [W]::Rect($hwnd)
      if ([Math]::Abs($a[0] - $x) -gt 10 -or [Math]::Abs($a[1] - $y) -gt 10 -or $r[0] -lt 0) {
        $discarded++   # the pointer moved, or the popup did not appear
        if ($log.Count -lt 12) { $log += "trial ${i}: vis=$($r[0]) predicted ($x,$y) actual ($($a[0]),$($a[1]))" }
      } else {
        $vis += $r[0]
        if ($r[1] -ge 0) { $paint += $r[1] } else { $noPaint++ }
        if ($shots -lt 3 -and $i -ge (5 + 10 * $shots)) {
          Save-Rect $a (Join-Path $shotDir ("first-frame-{0}.png" -f $shots)); $shots++
        }
      }
      Start-Sleep -Milliseconds 300
      [void][W]::Toggle($hwnd, $false, -1, -1, 0, 1500); Start-Sleep -Milliseconds 350
      if ([W]::IsWindowVisible($hwnd)) { [void][W]::Toggle($hwnd, $false, -1, -1, 0, 1500); Start-Sleep -Milliseconds 300 }
    }
    $res.trials_requested = $Trials; $res.trials_discarded = $discarded; $res.trials_without_paint = $noPaint
    $res.popup_size_px = @{ width = $popW; height = $popH }
    $res.window_visible_ms = Stats $vis
    $res.first_paint_ms = Stats $paint
    $res.discard_log = $log
    $res.note = "first-frames/*.png are screen grabs framed by the magenta backdrop, taken 20-40 ms after the first changed pixel"
  }
  finally {
    if ($app) { Stop-App $app }
    $backdrop.Close()
    [void][W]::SetCursorPos($origCursor[0], $origCursor[1])
  }
  $res.machine = Get-MachineInfo
  Save-Json $res "latency.json"
}

function Invoke-Screenshots {
  $script:ShotDir = Join-Path $OutDir "screenshots"; $null = New-Item -ItemType Directory -Force -Path $script:ShotDir
  $samples = @(
    "Meeting notes: ship the v0.1 installer on Friday",
    "https://example.com/docs/getting-started",
    "demo.user@example.com",
    "fn main() {`n    println!(`"hello, windy`");`n}",
    "INV-2026-00421",
    "C:\Users\demo\Documents\roadmap-q4.pdf",
    "The quick brown fox jumps over the lazy dog",
    "{`"theme`": `"dark`", `"historySize`": 200}",
    "Lunch at 12:30, table for four",
    "Draft: thanks for the quick review, merging after CI is green"
  )
  # Held in memory only; never printed; restored after the app is killed so the app does not record it.
  $savedClip = $null
  try { $savedClip = Get-Clipboard -Raw -ErrorAction Stop } catch { $savedClip = $null }
  $origCursor = [W]::Cursor(); $app = $null
  try {
    $app = Start-App $false
    $script:HwndPop = $app.Hwnd

    foreach ($s in $samples) { Set-Clipboard -Value $s; Start-Sleep -Milliseconds 900 }
    Start-Sleep -Milliseconds 800

    $px = 500; $py = 120
    $rc = Show-At $px $py
    $r = Grab $script:HwndPop "popup-items.png";  "popup-items.png    ok={0} {1}x{2}" -f $r.Ok, $r.Width, $r.Height
    Send-Keys "{DOWN}{DOWN}"
    $r = Grab $script:HwndPop "popup-selected.png"; "popup-selected.png ok={0}" -f $r.Ok
    Send-Keys "%p"
    $r = Grab $script:HwndPop "popup-pinned.png"; "popup-pinned.png   ok={0}" -f $r.Ok
    Send-Keys "inv"
    $r = Grab $script:HwndPop "popup-search.png"; "popup-search.png   ok={0}" -f $r.Ok
    Send-Keys "^u"

    # Settings window (Ctrl+,): report how long until real content appears.
    Send-Keys "^,"
    $clock = [Diagnostics.Stopwatch]::StartNew(); $hwndSet = [IntPtr]::Zero
    while ($hwndSet -eq [IntPtr]::Zero -and $clock.ElapsedMilliseconds -lt 5000) { $hwndSet = [W]::FindTopLevel([uint32]$app.Proc.Id, "Windy Settings"); Start-Sleep -Milliseconds 20 }
    if ($hwndSet -eq [IntPtr]::Zero) { throw "settings window did not open" }
    $contentAt = -1; $last = $null
    while ($clock.ElapsedMilliseconds -lt 15000) {
      $last = Grab $hwndSet $null
      if ($last.NonWhitePct -gt 50) { $contentAt = $clock.ElapsedMilliseconds; break }
      Start-Sleep -Milliseconds 250
    }
    if ($contentAt -ge 0) { "settings content first seen after $contentAt ms" } else { "settings content NOT seen within 15000 ms (the window is blank)" }
    Start-Sleep -Milliseconds 600
    $r = Grab $hwndSet "settings.png"; "settings.png ok={0} {1}x{2}" -f $r.Ok, $r.Width, $r.Height

    # Clear through the app's own "Clear all" (two clicks, 1 s apart) so nothing lingers in its database.
    $rc = Show-At $px $py
    Safe-Click $rc 376 676; Start-Sleep -Milliseconds 1000
    Safe-Click $rc 376 676
    $closed = Wait-Visible $script:HwndPop $false 3000
    "Clear all closed the popup: $closed"
    if (-not $closed) { throw "Clear all did not complete; history may still hold synthetic items" }
  }
  finally {
    if ($app) { Stop-App $app }
    if ($savedClip -is [string] -and $savedClip.Length -gt 0) { Set-Clipboard -Value $savedClip; "clipboard text restored" } else { "clipboard was empty or not text; nothing to restore" }
    [void][W]::SetCursorPos($origCursor[0], $origCursor[1])
  }
  "screenshots in: $($script:ShotDir)"
}

# Launch with a localhost-only WebView2 debugging port, open the settings window like a user would, and report each
# webview page's real state (URL, readiness, console errors). This is how a settings webview stuck on about:blank was found.
function Invoke-DebugProbe {
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=9333"
  $origCursor = [W]::Cursor(); $app = $null
  try {
    $app = Start-App $false
    $script:HwndPop = $app.Hwnd
    [void][W]::SetCursorPos(500, 120); Start-Sleep -Milliseconds 100
    [W]::Hotkey(); Start-Sleep -Milliseconds 1200
    Send-Keys "^,"
    $clock = [Diagnostics.Stopwatch]::StartNew(); $hwndSet = [IntPtr]::Zero
    while ($hwndSet -eq [IntPtr]::Zero -and $clock.ElapsedMilliseconds -lt 5000) { $hwndSet = [W]::FindTopLevel([uint32]$app.Proc.Id, "Windy Settings"); Start-Sleep -Milliseconds 20 }
    "settings window created: {0}" -f ($hwndSet -ne [IntPtr]::Zero)
    Start-Sleep -Seconds 6
    "--- playwright probe:"
    & node (Join-Path $script:Root "cdp-probe.cjs") 2>&1
  }
  finally {
    if ($app) { Stop-App $app }
    Remove-Item Env:\WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS -ErrorAction SilentlyContinue
    [void][W]::SetCursorPos($origCursor[0], $origCursor[1])
  }
}

switch ($Mode) {
  "startup-ram" { Invoke-StartupRam }
  "latency"     { Invoke-Latency }
  "screenshots" { Invoke-Screenshots }
  "debug-probe" { Invoke-DebugProbe }
}
