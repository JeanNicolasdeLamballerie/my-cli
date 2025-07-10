# $path_value = $HOME + "\code\projects\my-cli\target\release\my-cli.exe"
# function Invoke-Rust-dash
# {
#   Write-Output (Get-Date -Format HH:mm:ss.fff)
#   # TODO : handle stdout & stderr differently instead of crashing
#   $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
#   $output = & $path_value move $args
#   $stopwatch.Stop()
#   Write-Output (Get-Date -Format HH:mm:ss.fff)
#   Write-Output "Elapsed Time: $($stopwatch.Elapsed.TotalMilliseconds) ms"
#   Set-Location "$output"
# }
# function Invoke-Rust-dash2
# {
#   Write-Output (Get-Date -Format HH:mm:ss.fff)
#   # TODO : handle stdout & stderr differently instead of crashing
#   $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
#   # $job = Start-Job -ScriptBlock {
#   #   pwsh -NoProfile -Command `$(path_value) move $(args)`
#   # }
#   # Wait-Job $job
#   # $output = Receive-Job $job
# $dir = & $path_value
#   $stopwatch.Stop()
#   Write-Output (Get-Date -Format HH:mm:ss.fff)
#   Write-Output "Elapsed Time: $($stopwatch.Elapsed.TotalMilliseconds) ms"
#   Set-Location "$output"
# }
# function Invoke-Rust-dash3
# {
#   Write-Output (Get-Date -Format HH:mm:ss.fff)
#   # TODO : handle stdout & stderr differently instead of crashing
#   $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
#   $output = Invoke-Expression("$(path_value) move $(args)")
#   $stopwatch.Stop()
#   Write-Output (Get-Date -Format HH:mm:ss.fff)
#   Write-Output "Elapsed Time: $($stopwatch.Elapsed.TotalMilliseconds) ms"
#   Set-Location "$output"
# }
# #Set-Alias da $path_value move
#
# # DEV
# #
# #
# function Invoke-Rust-test
# {
#   # TODO : handle stdout & stderr differently instead of crashing
#   $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
#   $output = & test_process
#   $stopwatch.Stop()
#   Write-Output "Elapsed Time: $($stopwatch.Elapsed.TotalMilliseconds) ms"
#   Set-Location "$output"
# }
# function Invoke-Rust-cli
# {
#   # TODO : handle stdout & stderr differently instead of crashing
#   Write-Host "Running debug version.";
#   & $path_value $args
#
# }
#
#
# function Get-CliToolPort {
#     param (
#         [string]$FallbackPort = "5339"
#     )
#
#     if ($env:DEKHAREN_CLI_TOOL_PORT) {
#         return $env:DEKHAREN_CLI_TOOL_PORT
#     }
#
#     $tempDir = [System.IO.Path]::GetTempPath()
#     $portFile = Join-Path $tempDir "cli-daemon\port"
#
#     if (Test-Path $portFile) {
#         return (Get-Content $portFile -Raw).Trim()
#     }
#
#     return $FallbackPort
# }
#
# #
# #
# Set-Alias rushtest Invoke-Rust-test
# Set-Alias dash Invoke-Rust-dash
# Set-Alias tdash Invoke-Rust-dash2
# Set-Alias ttdash Invoke-Rust-dash3
# Set-Alias rush Invoke-Rust-cli
