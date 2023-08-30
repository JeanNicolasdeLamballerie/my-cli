$path = $HOME + "\code\projects\my-cli\target\debug\my-cli.exe"
function Invoke-Rust-mycli {
    # TODO : handle stdout & stderr differently instead of crashing
    Write-Host moving to "$args";
    $output = cmd /c $path move "$args" 2`>`&1
    Set-Location "$output"
}
Set-Alias dash Invoke-Rust-mycli
