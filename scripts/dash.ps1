$path = $HOME + "\code\projects\my-cli\target\debug\my-cli.exe"
function Invoke-Rust-dash {
    # TODO : handle stdout & stderr differently instead of crashing
    Write-Host moving to "$args";
    $output = cmd /c $path move "$args" 2`>`&1
    Set-Location "$output"
}


# DEV
#
#

function Invoke-Rust-cli {
    # TODO : handle stdout & stderr differently instead of crashing
    Write-Host "Running debug version.";
   cmd /c $path $args;

}

#
#
Set-Alias dash Invoke-Rust-dash
Set-Alias rush Invoke-Rust-cli
