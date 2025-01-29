$path = $HOME + "PATH_TO_EXE"
function Invoke-Rust-dash
{
  # TODO : handle stdout & stderr differently instead of crashing
  $output = Invoke-Expression("$path move $args 2`>`&1")
  Set-Location "$output"
}


# RELEASE
#
#

function Invoke-Rust-cli
{
  # TODO : handle stdout & stderr differently instead of crashing
  Invoke-Expression("$path $args");

}

#
#
Set-Alias dash Invoke-Rust-dash
Set-Alias rush Invoke-Rust-cli
