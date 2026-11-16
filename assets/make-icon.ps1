# Gera assets/icon.ico (multi-tamanho) e assets/icon.png (256px) do QuickEncoder.
# Uso: pwsh assets/make-icon.ps1
Add-Type -AssemblyName System.Drawing

function New-IconBitmap([int]$size) {
    $bmp = New-Object System.Drawing.Bitmap $size, $size, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = 'AntiAlias'
    $g.PixelOffsetMode = 'HighQuality'
    $g.Clear([System.Drawing.Color]::Transparent)
    $s = $size / 256.0

    # Fundo: quadrado arredondado escuro
    $r = 58 * $s
    $pad = 8 * $s
    $w = $size - 2 * $pad
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $d = 2 * $r
    $path.AddArc($pad, $pad, $d, $d, 180, 90)
    $path.AddArc($pad + $w - $d, $pad, $d, $d, 270, 90)
    $path.AddArc($pad + $w - $d, $pad + $w - $d, $d, $d, 0, 90)
    $path.AddArc($pad, $pad + $w - $d, $d, $d, 90, 90)
    $path.CloseFigure()
    $bgRect = New-Object System.Drawing.RectangleF 0, 0, $size, $size
    $bg = New-Object System.Drawing.Drawing2D.LinearGradientBrush $bgRect, ([System.Drawing.Color]::FromArgb(255, 38, 38, 46)), ([System.Drawing.Color]::FromArgb(255, 16, 16, 20)), 90.0
    $g.FillPath($bg, $path)

    # Símbolo: avanço rápido (dois triângulos) em gradiente laranja
    $fg = New-Object System.Drawing.Drawing2D.LinearGradientBrush $bgRect, ([System.Drawing.Color]::FromArgb(255, 255, 150, 60)), ([System.Drawing.Color]::FromArgb(255, 255, 84, 36)), 60.0
    function Tri([single]$x) {
        [System.Drawing.PointF[]]$pts = @(
            [System.Drawing.PointF]::new($x * $s, 78 * $s),
            [System.Drawing.PointF]::new(($x + 72) * $s, 128 * $s),
            [System.Drawing.PointF]::new($x * $s, 178 * $s)
        )
        $p = New-Object System.Drawing.Drawing2D.GraphicsPath
        $p.AddPolygon($pts)
        $pen = New-Object System.Drawing.Pen $fg, (14 * $s)
        $pen.LineJoin = 'Round'
        $g.DrawPath($pen, $p)
        $g.FillPath($fg, $p)
    }
    Tri 62
    Tri 128
    $g.Dispose()
    return $bmp
}

$dir = Split-Path -Parent $MyInvocation.MyCommand.Path
$sizes = 16, 24, 32, 48, 64, 256
$pngs = foreach ($sz in $sizes) {
    $bmp = New-IconBitmap $sz
    $ms = New-Object System.IO.MemoryStream
    $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    if ($sz -eq 256) { $bmp.Save((Join-Path $dir 'icon.png'), [System.Drawing.Imaging.ImageFormat]::Png) }
    $bmp.Dispose()
    , $ms.ToArray()
}

$out = New-Object System.IO.MemoryStream
$bw = New-Object System.IO.BinaryWriter $out
$bw.Write([uint16]0); $bw.Write([uint16]1); $bw.Write([uint16]$sizes.Count)
$offset = 6 + 16 * $sizes.Count
for ($i = 0; $i -lt $sizes.Count; $i++) {
    $sz = $sizes[$i]
    $b = if ($sz -ge 256) { 0 } else { $sz }
    $bw.Write([byte]$b); $bw.Write([byte]$b); $bw.Write([byte]0); $bw.Write([byte]0)
    $bw.Write([uint16]1); $bw.Write([uint16]32)
    $bw.Write([uint32]$pngs[$i].Length); $bw.Write([uint32]$offset)
    $offset += $pngs[$i].Length
}
foreach ($p in $pngs) { $bw.Write($p) }
$bw.Flush()
[System.IO.File]::WriteAllBytes((Join-Path $dir 'icon.ico'), $out.ToArray())
"icon.ico: $($out.Length) bytes"
