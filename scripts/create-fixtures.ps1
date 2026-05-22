param(
    [Parameter(Mandatory = $true)]
    [string]$Root
)

$ErrorActionPreference = "Stop"

# -------- 1. 安全拒绝 --------
if ([string]::IsNullOrWhiteSpace($Root)) {
    Write-Error "Root path cannot be empty"
    exit 1
}

$resolvedRoot = Resolve-Path -LiteralPath $Root -ErrorAction SilentlyContinue
if (-not $resolvedRoot) {
    $resolvedRoot = $Root
}

$resolvedRoot = [System.IO.Path]::GetFullPath($resolvedRoot).TrimEnd('\')

$dangerousPaths = @(
    [System.Environment]::GetFolderPath([System.Environment+SpecialFolder]::UserProfile),
    [System.Environment]::GetFolderPath([System.Environment+SpecialFolder]::Desktop),
    [System.Environment]::GetFolderPath([System.Environment+SpecialFolder]::Personal), # Documents
    [System.Environment]::GetFolderPath([System.Environment+SpecialFolder]::MyDocuments),
    [System.Environment]::GetFolderPath([System.Environment+SpecialFolder]::CommonDocuments),
    [System.Environment]::GetFolderPath([System.Environment+SpecialFolder]::ApplicationData)
)

# Also check common user roots
$additionalDangerous = @(
    [System.IO.Path]::Combine($env:USERPROFILE, "Downloads"),
    [System.IO.Path]::Combine($env:USERPROFILE, "Pictures"),
    [System.IO.Path]::Combine($env:USERPROFILE, "Videos"),
    [System.IO.Path]::Combine($env:USERPROFILE, "Music"),
    [System.IO.Path]::Combine($env:USERPROFILE, "Desktop"),
    [System.IO.Path]::Combine($env:USERPROFILE, "Documents")
)

foreach ($dangerPath in ($dangerousPaths + $additionalDangerous | Select-Object -Unique)) {
    if (-not [string]::IsNullOrWhiteSpace($dangerPath)) {
        $normalizedDanger = [System.IO.Path]::GetFullPath($dangerPath).TrimEnd('\')
        if ($resolvedRoot -eq $normalizedDanger) {
            Write-Error "Refusing to create fixtures in a dangerous system directory: $resolvedRoot"
            exit 1
        }
    }
}

# Create root directory
New-Item -ItemType Directory -Force -Path $resolvedRoot | Out-Null

# -------- 2. small-dir --------
$smallDir = Join-Path -Path $resolvedRoot -ChildPath "small-dir"
New-Item -ItemType Directory -Force -Path $smallDir | Out-Null

# Files
1..5 | ForEach-Object {
    New-Item -ItemType File -Force -Path (Join-Path -Path $smallDir -ChildPath "file_$_.txt") | Out-Null
}
# Subdirectories
"sub_empty", "sub_files" | ForEach-Object {
    New-Item -ItemType Directory -Force -Path (Join-Path -Path $smallDir -ChildPath $_) | Out-Null
}
1..3 | ForEach-Object {
    New-Item -ItemType File -Force -Path (Join-Path -Path $smallDir -ChildPath "sub_files\sub_file_$_.txt") | Out-Null
}
# Hidden file
$hiddenFilePath = Join-Path -Path $smallDir -ChildPath "hidden_file.txt"
Set-Content -LiteralPath $hiddenFilePath -Value "hidden file content" -NoNewline
Set-ItemProperty -LiteralPath $hiddenFilePath -Name Attributes -Value ([System.IO.FileAttributes]::Hidden) -ErrorAction SilentlyContinue

# -------- 3. large-10k-dir --------
$largeDir = Join-Path -Path $resolvedRoot -ChildPath "large-10k-dir"
New-Item -ItemType Directory -Force -Path $largeDir | Out-Null
1..10000 | ForEach-Object {
    New-Item -ItemType File -Force -Path (Join-Path -Path $largeDir -ChildPath "file_$_.txt") | Out-Null
}

# -------- 4. media-dir --------
$mediaDir = Join-Path -Path $resolvedRoot -ChildPath "media-dir"
New-Item -ItemType Directory -Force -Path $mediaDir | Out-Null
# Placeholder text files representing media files (no actual image/audio required)
"image_001.png", "image_002.jpg", "image_003.jpeg", "image_004.gif", "image_005.bmp", "image_006.webp" | ForEach-Object {
    New-Item -ItemType File -Force -Path (Join-Path -Path $mediaDir -ChildPath $_) | Out-Null
}
"video_001.mp4", "video_002.mkv", "video_003.avi", "video_004.mov" | ForEach-Object {
    New-Item -ItemType File -Force -Path (Join-Path -Path $mediaDir -ChildPath $_) | Out-Null
}
"audio_001.mp3", "audio_002.wav", "audio_003.flac", "audio_004.ogg" | ForEach-Object {
    New-Item -ItemType File -Force -Path (Join-Path -Path $mediaDir -ChildPath $_) | Out-Null
}
"doc_001.pdf", "doc_002.docx", "doc_003.xlsx" | ForEach-Object {
    New-Item -ItemType File -Force -Path (Join-Path -Path $mediaDir -ChildPath $_) | Out-Null
}

# -------- 5. conflict-source --------
$conflictSourceDir = Join-Path -Path $resolvedRoot -ChildPath "conflict-source"
New-Item -ItemType Directory -Force -Path $conflictSourceDir | Out-Null
"report.txt", "budget.xlsx", "photo.jpg", "readme.md", "archive.zip" | ForEach-Object {
    New-Item -ItemType File -Force -Path (Join-Path -Path $conflictSourceDir -ChildPath $_) | Out-Null
}
New-Item -ItemType Directory -Force -Path (Join-Path -Path $conflictSourceDir -ChildPath "project") | Out-Null
"main.rs", "lib.rs", "Cargo.toml" | ForEach-Object {
    New-Item -ItemType File -Force -Path (Join-Path -Path $conflictSourceDir -ChildPath "project\$_") | Out-Null
}

# -------- 6. conflict-target --------
$conflictTargetDir = Join-Path -Path $resolvedRoot -ChildPath "conflict-target"
New-Item -ItemType Directory -Force -Path $conflictTargetDir | Out-Null
# Same filenames as conflict-source for conflict testing
"report.txt", "budget.xlsx", "photo.jpg", "readme.md", "archive.zip" | ForEach-Object {
    New-Item -ItemType File -Force -Path (Join-Path -Path $conflictTargetDir -ChildPath $_) | Out-Null
}
# Extra files only in target
"notes.txt", "summary.pdf", "backup.zip" | ForEach-Object {
    New-Item -ItemType File -Force -Path (Join-Path -Path $conflictTargetDir -ChildPath $_) | Out-Null
}
New-Item -ItemType Directory -Force -Path (Join-Path -Path $conflictTargetDir -ChildPath "project") | Out-Null
# Different content in target's project dir (some same, some different)
"main.rs", "config.toml", "README.md" | ForEach-Object {
    New-Item -ItemType File -Force -Path (Join-Path -Path $conflictTargetDir -ChildPath "project\$_") | Out-Null
}
# An extra directory only in target
New-Item -ItemType Directory -Force -Path (Join-Path -Path $conflictTargetDir -ChildPath "extra-only-in-target") | Out-Null

# -------- 7. deep-tree --------
$deepTreeDir = Join-Path -Path $resolvedRoot -ChildPath "deep-tree"
New-Item -ItemType Directory -Force -Path $deepTreeDir | Out-Null

# Create a tree 5 levels deep with files at each level
$levels = @("level1", "level2", "level3", "level4", "level5")
$currentPath = $deepTreeDir

foreach ($level in $levels) {
    $currentPath = Join-Path -Path $currentPath -ChildPath $level
    New-Item -ItemType Directory -Force -Path $currentPath | Out-Null

    # A few files at each level
    1..3 | ForEach-Object {
        New-Item -ItemType File -Force -Path (Join-Path -Path $currentPath -ChildPath "file_at_$level`_$_.txt") | Out-Null
    }
}

# Also create a sparse side branch for additional testing
$branchPath = Join-Path -Path $deepTreeDir -ChildPath "level1\level2\side_branch"
New-Item -ItemType Directory -Force -Path $branchPath | Out-Null
1..5 | ForEach-Object {
    New-Item -ItemType File -Force -Path (Join-Path -Path $branchPath -ChildPath "side_file_$_.txt") | Out-Null
}

# A leaf directory with many files
$leafDir = Join-Path -Path $deepTreeDir -ChildPath "level1\level2\level3\level4\level5\leaf_bundle"
New-Item -ItemType Directory -Force -Path $leafDir | Out-Null
1..20 | ForEach-Object {
    New-Item -ItemType File -Force -Path (Join-Path -Path $leafDir -ChildPath "bundle_file_$_.txt") | Out-Null
}

# -------- 8. permission-cases (minimal placeholder) --------
$permDir = Join-Path -Path $resolvedRoot -ChildPath "permission-cases"
New-Item -ItemType Directory -Force -Path $permDir | Out-Null

# Create descriptive subdirectories for future permission test scenarios
"readonly-file", "no-access-dir", "nested-restricted", "normal-access" | ForEach-Object {
    New-Item -ItemType Directory -Force -Path (Join-Path -Path $permDir -ChildPath $_) | Out-Null
}

# Create placeholder files; actual ACL manipulation is deferred
"readonly-file\sample-readonly.txt", "no-access-dir\sample-blocked.txt", "normal-access\sample-normal.txt" | ForEach-Object {
    $filePath = Join-Path -Path $permDir -ChildPath $_
    New-Item -ItemType File -Force -Path $filePath | Out-Null
}

Write-Host "Fixtures created successfully under: $resolvedRoot" -ForegroundColor Green
exit 0
